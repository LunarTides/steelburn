use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{Level, log};
use tokio_tungstenite::tungstenite::Message;
use ulid::Ulid;

use crate::{
    Shared,
    game::{Game, SharedGame},
    netcode::{
        anticheat,
        net::{Packet, SharedPlayerInfo},
        peer_map::PeerMap,
    },
};

type GameData = (SharedGame, SocketAddr, SocketAddr);

#[derive(Default, Clone, Debug)]
pub struct Games {
    data: Shared<Vec<GameData>>,
}

impl Games {
    pub fn add(
        &self,
        peer_map: &PeerMap,
        peers: (
            (SocketAddr, SharedPlayerInfo),
            (SocketAddr, SharedPlayerInfo),
        ),
    ) -> SharedGame {
        // TODO: Randomize who is player 1 and who is player 2.
        let game = Game::new(peer_map.clone(), peers.0.0, peers.0.1, peers.1.0, peers.1.1);
        let shared_game = Arc::new(Mutex::new(game));
        self.data
            .lock()
            .unwrap()
            .push((shared_game.clone(), peers.0.0, peers.1.0));
        shared_game
    }

    pub fn remove(&self, peers: (SocketAddr, SocketAddr)) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        #[allow(clippy::suspicious_operation_groupings)]
        let queue_index = data
            .iter()
            .position(|x| x.1 == peers.0 || x.1 == peers.1 || x.2 == peers.0 || x.2 == peers.1)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn remove_by_peer(&self, peer: SocketAddr) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        let queue_index = data.iter().position(|x| x.1 == peer || x.2 == peer)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn is_poisoned(&self) -> bool {
        self.data.is_poisoned()
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` here."
    )]
    pub fn get_opposing_peer(&self, peer: SocketAddr) -> Option<SocketAddr> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        let opposing_peer = if peers.1 == peer { peers.2 } else { peers.1 };
        Some(opposing_peer)
    }

    /// Send a message to all peers connected to the same game as `peer`.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` nor `map_peers` here."
    )]
    pub fn send_message_by_peer(
        &self,
        peer_map: &PeerMap,
        peer: SocketAddr,
        message: Message,
    ) -> Option<()> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;

        // TODO: Is unwrap safe here?
        let map_peers = peer_map.get().unwrap();
        let sinks = map_peers
            .iter()
            .filter(|(addr, _)| *addr == &peers.1 || *addr == &peers.2)
            .map(|(_, (_, sink))| sink);

        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }

        Some(())
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` here."
    )]
    pub fn get_game_from_peer(&self, peer: SocketAddr) -> Option<SharedGame> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        Some(peers.0.clone())
    }

    pub fn peer_in_game(&self, peer: SocketAddr) -> bool {
        let game_peers = self.data.lock().unwrap();
        game_peers.iter().any(|x| x.1 == peer || x.2 == peer)
    }

    // TODO: Return a result.
    // TODO: Split into multiple functions.
    pub fn handle_client_packet(&self, peer_map: &PeerMap, peer: SocketAddr, mut packet: Packet) {
        let Some(game) = self.get_game_from_peer(peer) else {
            log!(Level::Error, "Could not get game from peer.");
            return;
        };

        if !anticheat::check(&game, peer_map, peer, &packet) {
            return;
        }

        if packet.packet_type == "end_turn" {
            let mut game_lock = game.lock().unwrap();
            log!(
                Level::Debug,
                "Ending turn {} ({})",
                game_lock.turn,
                game_lock
                    .get_current_player()
                    .lock()
                    .unwrap()
                    .info
                    .lock()
                    .unwrap()
                    .username
            );
            game_lock.end_turn();

            // Repeat packet to both clients.
            let player = game_lock.get_current_player();
            let player_lock = player.lock().unwrap();
            drop(game_lock);

            packet.prepare_for_response(!player_lock.is_starting_player);
            packet.data.insert(
                "opponent".into(),
                serde_json::to_value(player_lock.clone()).unwrap(),
            );
            self.send_message_by_peer(peer_map, peer, packet.into());
            return;
        }
        if packet.packet_type == "play_card" {
            let Some(hash_value) = packet.data.get("hash") else {
                log!(
                    Level::Error,
                    "'play_card' packet doesn't have a 'hash' data field."
                );
                return;
            };
            let Some(hash_string) = hash_value.as_str() else {
                log!(
                    Level::Error,
                    "'play_card' packet 'hash' data field is not a string."
                );
                return;
            };
            let hash_ulid = match Ulid::from_string(hash_string) {
                Ok(hash_field) => hash_field,
                Err(err) => {
                    log!(
                        Level::Error,
                        "'play_card' packet 'hash' data field is not proper Ulid. {err}"
                    );
                    return;
                }
            };
            log!(Level::Debug, "Playing '{hash_value}'.");

            let game_lock = game.lock().unwrap();

            let is_player_turn = game_lock.current_player_peer == peer;
            if !is_player_turn {
                // TODO: Send this to client.
                log!(Level::Error, "Not this player's turn.");
                return;
            }

            let player = game_lock.get_player_from_peer(peer).unwrap();
            let mut player_lock = player.lock().unwrap();

            let card = match player_lock.play_card(&game_lock, hash_ulid) {
                Err(err) => {
                    log!(Level::Error, "{err}");
                    return;
                }
                Ok(card) => card,
            };

            log!(
                Level::Debug,
                "Board for {}: {:?} ({})",
                player_lock.peer,
                player_lock.board,
                player_lock.board.len()
            );
            drop(player_lock);

            let card_lock = card.lock().unwrap();

            // Run the `onplay` function on the lua side.
            if let Some(onplay) = &card_lock.onplay {
                game_lock
                    .lua_vm
                    .prepare_for_card(player.clone().into())
                    .unwrap();
                // FIXME: This will panic on lua runtime error.
                onplay.call::<()>(()).unwrap();
            }

            let mut clone = card_lock.clone();
            clone.hash = card_lock.hash;
            drop(card_lock);

            let player_lock = player.lock().unwrap();
            // Repeat packet to both clients.
            // TODO: Add a function to automate this kind of thing.
            packet.prepare_for_response(player_lock.is_starting_player);
            packet
                .data
                .insert("card".into(), serde_json::to_value(clone).unwrap());
            packet.data.insert(
                "player".into(),
                serde_json::to_value(player_lock.clone()).unwrap(),
            );

            self.send_message_by_peer(peer_map, peer, packet.into());
            return;
        }

        // TODO: Replace with something other than echo packet :)
        let json = r#"
            {
                "type": "echo",
                "data": {
                    "from_peer": "replace me",
                    "from_player": "replace me",
                    "packet": "replace me",
                    "game_id": "replace me"
                }
            }
        "#;
        let mut response_packet: Packet = serde_json::from_str(json).unwrap();
        response_packet.data.insert(
            "from_peer".into(),
            serde_json::Value::String(peer.to_string()),
        );
        let game_lock = game.lock().unwrap();
        response_packet.data.insert(
            "from_player".into(),
            serde_json::Value::String(
                game_lock
                    .get_player_from_peer(peer)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .info
                    .lock()
                    .unwrap()
                    .username
                    .clone(),
            ),
        );
        response_packet.data.insert(
            "packet".into(),
            serde_json::Value::String(packet.packet_type.clone()),
        );
        response_packet.data.insert(
            "game_id".into(),
            serde_json::Value::String(game_lock.id.to_string()),
        );
        drop(game_lock);

        self.send_message_by_peer(peer_map, peer, response_packet.into());
    }
}
