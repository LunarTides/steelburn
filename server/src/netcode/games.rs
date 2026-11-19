use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio_tungstenite::tungstenite::Message;

use crate::{
    Shared,
    game::{Game, SharedGame},
    netcode::{
        net::{SharedPlayerInfo, Tx},
        peer_map::PeerMap,
    },
    pack::PackCollection,
};

type GameData = (SharedGame, SocketAddr, SocketAddr);

#[derive(Default, Clone, Debug)]
pub struct Games {
    data: Shared<Vec<GameData>>,
}

impl Games {
    pub fn add(
        &self,
        packs: &PackCollection,
        peer_map: &PeerMap,
        player1_peer: SocketAddr,
        player1_sink: Tx,
        player1_info: SharedPlayerInfo,
        player2_peer: SocketAddr,
        player2_sink: Tx,
        player2_info: SharedPlayerInfo,
    ) -> SharedGame {
        // TODO: Randomize who is player 1 and who is player 2.
        let game = Game::new(
            packs.clone(),
            peer_map.clone(),
            player1_peer,
            player1_sink,
            player1_info,
            player2_peer,
            player2_sink,
            player2_info,
        );
        let shared_game = Arc::new(Mutex::new(game));
        self.data
            .lock()
            .unwrap()
            .push((shared_game.clone(), player1_peer, player2_peer));
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
}
