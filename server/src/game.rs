use std::{net::SocketAddr, sync::Mutex};

use log::{Level, log};
use ulid::Ulid;

use crate::{
    Shared,
    card_collection::CardCollection,
    lua::LuaVM,
    netcode::{
        net::{SharedPlayerInfo, Tx},
        packet::ResponsePacket,
        peer_map::PeerMap,
    },
    pack::Pack,
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
    // TODO: Shouldn't be global. Use individual lua VMs instead.
    pub lua_vm: LuaVM,
    pub peer_map: PeerMap,
}

impl Game {
    pub fn new(
        peer_map: PeerMap,
        player1_peer: SocketAddr,
        player1_sink: Tx,
        player1_info: SharedPlayerInfo,
        player2_peer: SocketAddr,
        player2_sink: Tx,
        player2_info: SharedPlayerInfo,
    ) -> Self {
        let id = Ulid::generate();

        let player1 = SharedPlayer::new(Mutex::new(Player::new(
            id,
            player1_peer,
            player1_sink,
            true,
            player1_info,
        )));
        let player2 = SharedPlayer::new(Mutex::new(Player::new(
            id,
            player2_peer,
            player2_sink,
            false,
            player2_info,
        )));

        player1
            .lock()
            .unwrap()
            .set_opponent(player2.clone(), player2_peer);
        player2
            .lock()
            .unwrap()
            .set_opponent(player1.clone(), player1_peer);

        Self {
            player1,
            player1_peer,
            player2,
            player2_peer,
            id,
            turn: 0,
            current_player_peer: player1_peer,
            lua_vm: LuaVM::start().unwrap(),
            peer_map,
        }
    }

    /// Returns the player associated with a peer. (IP-address of the WebSocket client.)
    pub fn get_player_from_peer(&self, peer: SocketAddr) -> Option<SharedPlayer> {
        if self.player1_peer == peer {
            return Some(self.player1.clone());
        } else if self.player2_peer == peer {
            return Some(self.player2.clone());
        }

        None
    }

    /// Returns if the peer is the starting player.
    pub fn is_peer_starting_player(&self, peer: SocketAddr) -> bool {
        self.player1_peer == peer
    }

    /// Sends a packet to both connected clients.
    /// This locks both player mutexes.
    /// This locks all cards in the players' hand, deck, board, etc.
    pub fn send_packet(&self, mut packet: ResponsePacket) {
        let peers = [self.player1_peer, self.player2_peer];

        // TODO: Is unwrap safe here?
        let map_peers = self.peer_map.get().unwrap();
        let mappings = map_peers
            .iter()
            .filter(|(addr, _)| peers.contains(addr))
            .map(|(addr, (_, sink))| (sink, self.get_player_from_peer(*addr)));

        for (sink, player) in mappings {
            // TODO: Is unwrap safe here?
            packet.send_to_option_shared(sink, player).unwrap();
        }
    }

    pub fn start_game(&self, response: &mut Option<&mut ResponsePacket>) {
        let mut player1 = self.player1.lock().unwrap();
        player1.set_empty_mana(1);
        player1.set_mana(1);

        player1.draw_cards(3, response);
        // NOTE: Draw cards requires the opposing player to be free.
        drop(player1);

        let mut player2 = self.player2.lock().unwrap();
        player2.draw_cards(4, response);

        // Create the coin.
        // TODO: Don't rely on epsilon.
        // TODO: Don't re-import the pack. Store the packs globally.
        // TODO: The coin here might be linked.
        let pack = Pack::import(self, "../packs/@steelburn/epsilon").unwrap();
        let the_coin = pack.cards.get_with_id("The Coin").unwrap();
        player2.add_to_hand(the_coin);
        // TODO: This probably shouldn't be sent as a draw card packet?
        if let Some(response) = response {
            response.send_draw_cards_packet(
                &mut player2,
                &CardCollection::from_vec(vec![the_coin.clone()]),
            );
        }

        let mut player1 = self.player1.lock().unwrap();
        // let mut player2 = self.player2.lock().unwrap();

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

        current_player_lock.add_empty_mana(1);
        // TODO: Apply overload.
        let new_mana = current_player_lock.empty_mana;
        current_player_lock.set_mana(new_mana);
        current_player_lock.draw_cards(1, response);

        // let binding = self.get_opposing_player();
        // let mut opposing_player = binding.lock().unwrap();
        // drop(current_player);
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
