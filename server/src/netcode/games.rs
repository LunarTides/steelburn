use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_util::future::Shared;
use log::{Level, log};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    game::{Game, SharedGame},
    netcode::{
        anticheat,
        net::{Packet, PeerMap, SharedPlayerInfo},
    },
};

type GameData = (SharedGame, SocketAddr, SocketAddr);

#[derive(Default, Clone)]
pub struct Games {
    // TODO: This should be the current ongoing games between two peers.
    pub data: Arc<Mutex<Vec<GameData>>>,
}

impl Games {
    pub fn add(
        &self,
        peers: (
            (SocketAddr, SharedPlayerInfo),
            (SocketAddr, SharedPlayerInfo),
        ),
    ) -> SharedGame {
        // TODO: Randomize who is player 1 and who is player 2.
        let game = Game::new(peers.0.0, peers.0.1, peers.1.0, peers.1.1);
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
        message: &Message,
    ) -> Option<()> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;

        let map_peers = peer_map.lock().unwrap();
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

    pub fn handle_client_packet(&self, peer_map: &PeerMap, peer: SocketAddr, packet: &Packet) {
        let Some(game) = self.get_game_from_peer(peer) else {
            log!(Level::Error, "Could not get game from peer.");
            return;
        };

        if !anticheat::check(&game, peer_map, peer, packet) {
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
            drop(game_lock);

            // Repeat packet to both clients.
            self.send_message_by_peer(
                peer_map,
                peer,
                &Message::Text(serde_json::to_string(&packet).unwrap().into()),
            );
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

        self.send_message_by_peer(
            peer_map,
            peer,
            &Message::Text(serde_json::to_string(&response_packet).unwrap().into()),
        );
    }
}
