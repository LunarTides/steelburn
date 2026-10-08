use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use tokio_tungstenite::tungstenite::Message;

use crate::{
    game::Game,
    netcode::net::{Packet, PeerMap},
};

#[derive(Default, Clone)]
pub struct Games {
    // TODO: This should be the current ongoing games between two peers.
    pub data: Arc<Mutex<Vec<(Arc<Game>, SocketAddr, SocketAddr)>>>,
}

impl Games {
    pub fn add(&self, peers: (SocketAddr, SocketAddr)) {
        let game = Game::new("en".into(), "to".into());
        self.data
            .lock()
            .unwrap()
            .push((Arc::new(game), peers.0, peers.1));
    }

    pub fn remove(&self, peers: (SocketAddr, SocketAddr)) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
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

    pub fn get_opposing_peer(&self, peer: SocketAddr) -> Option<SocketAddr> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        let opposing_peer = if peers.1 == peer { peers.2 } else { peers.1 };
        Some(opposing_peer)
    }

    /// Send a message to all peers connected to the same game as `peer`.
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
            .map(|(_, sink)| sink);

        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }

        Some(())
    }

    pub fn get_game_from_peer(&self, peer: SocketAddr) -> Option<Arc<Game>> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        Some(peers.0.clone())
    }

    pub fn handle_client_packet(&self, peer_map: &PeerMap, peer: SocketAddr, packet: Packet) {
        println!("Handling packet {packet:?} from {peer}");

        let game = match self.get_game_from_peer(peer) {
            Some(game) => game,
            None => {
                println!("Error: Could not get game from peer.");
                return;
            }
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
        let mut packet: Packet = serde_json::from_str(json).unwrap();
        packet.data.insert(
            "game_id".into(),
            serde_json::Value::String(game.id.to_string()),
        );

        self.send_message_by_peer(
            peer_map,
            peer,
            &Message::Text(serde_json::to_string(&packet).unwrap().into()),
        );
    }
}
