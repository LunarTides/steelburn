use std::{net::SocketAddr, sync::Mutex};

use log::{Level, log};
use tokio_tungstenite::tungstenite::Message;
use ulid::Ulid;

use crate::{
    Shared,
    lua::LuaVM,
    netcode::{
        net::{Packet, ResponsePacket, SharedPlayerInfo},
        peer_map::PeerMap,
    },
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

    pub fn send_packet(&self, packet: Packet) {
        let peers = [self.player1_peer, self.player2_peer];

        // TODO: Is unwrap safe here?
        let map_peers = self.peer_map.get().unwrap();
        let sinks = map_peers
            .iter()
            .filter(|(addr, _)| peers.contains(addr))
            .map(|(_, (_, sink))| sink);

        let message: Message = packet.into();
        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }
    }

    pub fn send_packet_to(&self, peer: SocketAddr, packet: ResponsePacket) -> Option<()> {
        // TODO: Is unwrap safe here?
        let map_peers = self.peer_map.get().unwrap();
        let sink = &map_peers.iter().find(|(addr, _)| *addr == &peer)?.1.1;

        let message: Message = packet.into();
        sink.unbounded_send(message).unwrap();

        Some(())
    }

    pub fn start_game(&self, response: &mut Option<&mut ResponsePacket>) {
        let mut player1 = self.player1.lock().unwrap();
        player1.draw_cards(self, 3, response);
        // NOTE: Draw cards requires the opposing player to be free.
        drop(player1);

        let mut player2 = self.player2.lock().unwrap();
        player2.draw_cards(self, 4, response);

        let mut player1 = self.player1.lock().unwrap();

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

        let binding = self.get_current_player();
        let mut current_player = binding.lock().unwrap();

        // TODO: Cap to empty mana.
        current_player.add_mana(1);
        current_player.draw_cards(self, 1, response);

        // let binding = self.get_opposing_player();
        // let mut opposing_player = binding.lock().unwrap();
        drop(current_player);
        // drop(opposing_player);
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
