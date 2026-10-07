use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio_tungstenite::tungstenite::Message;

use crate::netcode::net::PeerMap;

#[derive(Default, Clone)]
pub struct Games {
    // TODO: This should be the current ongoing games between two peers.
    pub data: Arc<Mutex<Vec<(SocketAddr, SocketAddr)>>>,
}

impl Games {
    pub fn add(&self, peers: (SocketAddr, SocketAddr)) {
        self.data.lock().unwrap().push(peers);
    }

    pub fn remove(&self, peers: (SocketAddr, SocketAddr)) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        let queue_index = data.iter().position(|x| *x == peers)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn remove_by_peer(&self, peer: SocketAddr) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        let queue_index = data.iter().position(|x| x.0 == peer || x.1 == peer)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    /// Send a message to all peers connected to the same game as `peer`.
    pub fn send_message_by_peer(
        &self,
        peer_map: &PeerMap,
        peer: SocketAddr,
        message: &Message,
    ) -> Option<()> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.0 == peer || x.1 == peer)?;

        let map_peers = peer_map.lock().unwrap();
        let sinks = map_peers
            .iter()
            .filter(|(addr, _)| *addr == &peers.0 || *addr == &peers.1)
            .map(|(_, sink)| sink);

        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }

        Some(())
    }
}
