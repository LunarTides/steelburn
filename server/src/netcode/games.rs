use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{Level, log};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    game::Game,
    netcode::net::{Packet, PeerMap, SharedPlayerInfo},
};

type GameData = (Arc<Game>, SocketAddr, SocketAddr);

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
    ) {
        // TODO: Randomize who is player 1 and who is player 2.
        let game = Game::new(peers.0.0, peers.0.1, peers.1.0, peers.1.1);
        self.data
            .lock()
            .unwrap()
            .push((Arc::new(game), peers.0.0, peers.1.0));
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
    pub fn get_game_from_peer(&self, peer: SocketAddr) -> Option<Arc<Game>> {
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
            log!(Level::Error, "Error: Could not get game from peer.");
            return;
        };

        let json = r#"
            {
                "type": "test",
                "data": {
                    "message": "some packet sent",
                    "game_id": "replace me"
                }
            }
        "#;
        let mut response_packet: Packet = serde_json::from_str(json).unwrap();
        response_packet.data.insert(
            "game_id".into(),
            serde_json::Value::String(game.id.to_string()),
        );

        self.send_message_by_peer(
            peer_map,
            peer,
            &Message::Text(serde_json::to_string(&response_packet).unwrap().into()),
        );
    }
}
