use std::{net::SocketAddr, sync::Mutex};

use log::{Level, log};
use ulid::Ulid;

use crate::{
    Shared,
    card_collection::CardCollection,
    lua::LuaVM,
    netcode::{net::SharedPlayerInfo, packet::ResponsePacket, peer_map::PeerMap},
    player::{Player, SharedPlayer},
};

pub type SharedGame = Shared<Game>;

#[derive(Debug)]
pub struct Game {
    pub id: Ulid,
    pub player1: SharedPlayer,
    pub player1_peer: SocketAddr,
    pub player2: SharedPlayer,
    pub player2_peer: SocketAddr,
    pub turn: u64,
    pub current_player_peer: SocketAddr,
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
            current_player_peer: player1_peer,
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

    pub fn send_packet(&self, mut packet: ResponsePacket) {
        packet.use_player_changes_shared(&self.player1);
        packet.use_player_changes_shared(&self.player2);

        let peers = [self.player1_peer, self.player2_peer];

        // TODO: Is unwrap safe here?
        let map_peers = self.peer_map.get().unwrap();
        let mappings = map_peers
            .iter()
            .filter(|(addr, _)| peers.contains(addr))
            .map(|(addr, (_, sink))| (sink, self.get_player_from_peer(*addr)));

        for (sink, player) in mappings {
            // TODO: Is unwrap safe here?
            packet.send_to(sink, player.as_ref()).unwrap();
        }
    }

    pub fn send_packet_to(
        &self,
        peer: SocketAddr,
        packet: &mut ResponsePacket,
        player: Option<&SharedPlayer>,
    ) -> Option<()> {
        // TODO: Is unwrap safe here?
        let map_peers = self.peer_map.get().unwrap();
        let sink = &map_peers.iter().find(|(addr, _)| *addr == &peer)?.1.1;

        packet.send_to(sink, player).unwrap();
        Some(())
    }

    pub fn start_game(&self, response: &mut Option<&mut ResponsePacket>) {
        let mut player1 = self.player1.lock().unwrap();
        let cards = player1.draw_cards(3);
        // NOTE: Draw cards requires the opposing player to be free.
        drop(player1);
        self.send_draw_cards_packet(response, &self.player1, &cards);

        let mut player2 = self.player2.lock().unwrap();
        let cards = player2.draw_cards(4);
        drop(player2);
        self.send_draw_cards_packet(response, &self.player2, &cards);

        let mut player1 = self.player1.lock().unwrap();
        let mut player2 = self.player2.lock().unwrap();

        player1.mana = 1;

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

    pub fn end_turn(&mut self, response: &mut Option<&mut ResponsePacket>) {
        self.turn = self.turn.saturating_add(1);
        self.current_player_peer = if self.turn.is_multiple_of(2) {
            self.player1_peer
        } else {
            self.player2_peer
        };

        let current_player = self.get_current_player();
        let mut current_player_lock = current_player.lock().unwrap();

        // TODO: Cap to empty mana.
        current_player_lock.add_mana(1);
        let cards = current_player_lock.draw_cards(1);
        drop(current_player_lock);
        self.send_draw_cards_packet(response, &current_player, &cards);

        // let binding = self.get_opposing_player();
        // let mut opposing_player = binding.lock().unwrap();
        // drop(current_player);
        // drop(opposing_player);
    }

    pub fn send_draw_cards_packet(
        &self,
        response: &mut Option<&mut ResponsePacket>,
        player: &SharedPlayer,
        cards: &CardCollection,
    ) {
        if response.is_none() {
            return;
        }

        // Send cards to clients.
        let response = response.as_mut().unwrap();

        let mut friendly_response = response.clone();
        friendly_response.packet_type = "draw_cards".into();
        friendly_response.add_cards("cards", cards.clone());
        let mut player_lock = player.lock().unwrap();
        friendly_response.is_starting_player = player_lock.is_starting_player;
        friendly_response.use_player_changes(&mut player_lock);
        friendly_response.use_player_changes_shared(&player_lock.get_opponent(self));
        let player_peer = player_lock.peer;
        drop(player_lock);
        self.send_packet_to(player_peer, &mut friendly_response, Some(player));

        // Opponent
        let mut opponent_response = response.clone();
        opponent_response.packet_type = "opponent_draw_cards".into();
        opponent_response
            .response_data
            .insert("amount".into(), cards.len().into());
        let mut player_lock = player.lock().unwrap();
        opponent_response.is_starting_player = player_lock.is_starting_player;
        opponent_response.use_player_changes(&mut player_lock);
        opponent_response.use_player_changes_shared(&player_lock.get_opponent(self));
        self.send_packet_to(
            player_lock.get_opponent_peer(self),
            &mut opponent_response,
            Some(&player_lock.get_opponent(self)),
        );
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
