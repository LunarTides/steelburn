use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{Level, log};
use tokio_tungstenite::tungstenite::Message;
use ulid::Ulid;

use crate::{
    lua::LuaVM,
    netcode::net::{Packet, PeerMap, SharedPlayerInfo},
    player::{Player, SharedPlayer},
};

pub type SharedGame = Arc<Mutex<Game>>;

#[derive(Debug)]
pub struct Game {
    pub id: Ulid,
    pub player1: SharedPlayer,
    pub player1_peer: SocketAddr,
    pub player2: SharedPlayer,
    pub player2_peer: SocketAddr,
    pub turn: u64,
    pub lua_vm: LuaVM,
    pub peer_map: PeerMap,
}

impl Game {
    pub fn new(
        peer_map: PeerMap,
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
            player1_peer,
            player2: SharedPlayer::new(Mutex::new(Player::new(
                id,
                player2_peer,
                false,
                player2_info,
            ))),
            player2_peer,
            id,
            turn: 0,
            lua_vm: LuaVM::start().unwrap(),
            peer_map,
        }
    }

    pub fn get_player_from_peer(&self, peer: SocketAddr) -> Option<SharedPlayer> {
        if self.player1_peer == peer {
            return Some(self.player1.clone());
        } else if self.player2_peer == peer {
            return Some(self.player2.clone());
        }

        None
    }

    pub fn is_peer_starting_player(&self, peer: SocketAddr) -> bool {
        self.player1_peer == peer
    }

    pub fn send_packet(&self, packet: Packet) {
        let peers = [self.player1_peer, self.player2_peer];

        let map_peers = self.peer_map.lock().unwrap();
        let sinks = map_peers
            .iter()
            .filter(|(addr, _)| peers.contains(addr))
            .map(|(_, (_, sink))| sink);

        let message: Message = packet.into();
        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }
    }

    pub fn send_packet_to(&self, peer: SocketAddr, packet: Packet) -> Option<()> {
        let map_peers = self.peer_map.lock().unwrap();
        let sink = &map_peers.iter().find(|(addr, _)| *addr == &peer)?.1.1;

        let message: Message = packet.into();
        sink.unbounded_send(message).unwrap();

        Some(())
    }

    pub fn start_game(&self) {
        let mut player1 = self.player1.lock().unwrap();
        let mut player2 = self.player2.lock().unwrap();

        for player in [&mut player1, &mut player2] {
            let draw_amount = if player.is_starting_player { 3 } else { 4 };
            player.draw_cards(self, draw_amount);
        }

        log!(
            Level::Debug,
            "Hand Sizes: ({}, {})",
            player1.hand.len(),
            player2.hand.len()
        );
        log!(
            Level::Debug,
            "Deck Sizes: ({}, {})",
            player1.deck.len(),
            player2.deck.len()
        );
    }

    pub fn end_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);

        let binding = self.get_current_player();
        let mut current_player = binding.lock().unwrap();
        let binding = self.get_opposing_player();
        let mut opposing_player = binding.lock().unwrap();

        current_player.mana = current_player.mana.saturating_add(1);
        current_player.draw_cards(self, 1);
        drop(current_player);
        drop(opposing_player);

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
