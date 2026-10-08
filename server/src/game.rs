use std::{net::SocketAddr, sync::Arc};

use ulid::Ulid;

use crate::{netcode::net::SharedPlayerInfo, player::Player};

#[derive(Debug)]
pub struct Game {
    pub id: Ulid,
    pub player1: Arc<Player>,
    pub player2: Arc<Player>,
    pub turn: u64,
}

impl Game {
    pub fn new(
        player1_peer: SocketAddr,
        player1_info: SharedPlayerInfo,
        player2_peer: SocketAddr,
        player2_info: SharedPlayerInfo,
    ) -> Self {
        let id = Ulid::generate();
        Self {
            player1: Arc::new(Player::new(id, player1_peer, true, player1_info)),
            player2: Arc::new(Player::new(id, player2_peer, false, player2_info)),
            id,
            turn: 0,
        }
    }

    pub fn get_player_from_peer(&self, peer: SocketAddr) -> Option<Arc<Player>> {
        if self.player1.peer == peer {
            return Some(self.player1.clone());
        } else if self.player2.peer == peer {
            return Some(self.player2.clone());
        }

        None
    }

    pub const fn end_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);
    }

    pub fn get_current_player(&self) -> Arc<Player> {
        if self.turn.is_multiple_of(2) {
            self.player1.clone()
        } else {
            self.player2.clone()
        }
    }

    pub fn get_opposing_player(&self) -> Arc<Player> {
        if self.turn.is_multiple_of(2) {
            self.player2.clone()
        } else {
            self.player1.clone()
        }
    }
}
