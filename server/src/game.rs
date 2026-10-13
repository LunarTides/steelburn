use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{Level, log};
use ulid::Ulid;

use crate::{
    lua::LuaVM,
    netcode::net::SharedPlayerInfo,
    player::{Player, SharedPlayer},
};

pub type SharedGame = Arc<Mutex<Game>>;

#[derive(Debug)]
pub struct Game {
    pub id: Ulid,
    pub player1: SharedPlayer,
    pub player2: SharedPlayer,
    pub turn: u64,
    pub lua_vm: LuaVM,
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
            player1: SharedPlayer::new(Mutex::new(Player::new(
                id,
                player1_peer,
                true,
                player1_info,
            ))),
            player2: SharedPlayer::new(Mutex::new(Player::new(
                id,
                player2_peer,
                false,
                player2_info,
            ))),
            id,
            turn: 0,
            lua_vm: LuaVM::start().unwrap(),
        }
    }

    pub fn get_player_from_peer(&self, peer: SocketAddr) -> Option<SharedPlayer> {
        if self.player1.lock().unwrap().peer == peer {
            return Some(self.player1.clone());
        } else if self.player2.lock().unwrap().peer == peer {
            return Some(self.player2.clone());
        }

        None
    }

    pub fn end_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);

        // TODO: Remove. Temporarily call 'onplay' on a Sheep in each player's deck.
        let player1_lock = self.player1.lock().unwrap();
        if !player1_lock
            .info
            .lock()
            .unwrap()
            .username
            .ends_with("(Lua)")
        {
            let card_clone = player1_lock.deck.get_with_id("Sheep").unwrap().clone();
            let card_lock = card_clone.lock().unwrap();
            drop(player1_lock);

            if let Some(onplay) = &card_lock.onplay {
                self.lua_vm.prepare_for_card(self, &card_lock).unwrap();
                // NOTE: This will panic on lua runtime error.
                onplay.call::<()>(()).unwrap();
            }
            drop(card_lock);

            let player2_lock = self.player2.lock().unwrap();
            let card_clone = player2_lock.deck.get_with_id("Sheep").unwrap().clone();
            let card_lock = card_clone.lock().unwrap();
            drop(player2_lock);

            if let Some(onplay) = &card_lock.onplay {
                self.lua_vm.prepare_for_card(self, &card_lock).unwrap();
                onplay.call::<()>(()).unwrap();
            }
            drop(card_lock);

            let player1_lock = self.player1.lock().unwrap();
            log!(
                Level::Debug,
                "Player 1's name was changed to: {}",
                player1_lock.info.lock().unwrap().username
            );
        }
    }

    pub fn get_current_player(&self) -> SharedPlayer {
        if self.turn.is_multiple_of(2) {
            self.player1.clone()
        } else {
            self.player2.clone()
        }
    }

    pub fn get_opposing_player(&self) -> SharedPlayer {
        if self.turn.is_multiple_of(2) {
            self.player2.clone()
        } else {
            self.player1.clone()
        }
    }
}
