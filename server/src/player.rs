use std::{
    error::Error,
    fmt::Display,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use mlua::prelude::*;
use serde_json::Map;
use ulid::Ulid;

use crate::{
    card::{Card, SharedCard},
    card_collection::CardCollection,
    game::{Game, SharedGame},
    netcode::net::{Packet, SharedPlayerInfo},
};

pub type SharedPlayer = Arc<Mutex<Player>>;

#[derive(Debug, Clone)]
pub struct Player {
    // TODO: Replace with a SharedGame reference.
    pub game_id: Ulid,
    pub peer: SocketAddr,
    #[allow(
        clippy::struct_field_names,
        reason = "Without the 'player' at the end, it would be a bit unclear what this field is used for."
    )]
    pub is_starting_player: bool,
    pub info: SharedPlayerInfo,
    pub mana: u32,
    pub empty_mana: u32,
    pub max_mana: u32,
    pub deck: CardCollection,
    pub hand: CardCollection,
    pub board: CardCollection,
}

impl LuaUserData for Player {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("is_starting_player", |_, this| Ok(this.is_starting_player));

        fields.add_field_method_get("username", |_, this| {
            Ok(this.info.lock().unwrap().username.clone())
        });
        fields.add_field_method_set("username", |_, this, val| {
            this.info.lock().unwrap().username = val;
            Ok(())
        });
    }
}

impl Player {
    pub fn new(
        game_id: Ulid,
        peer: SocketAddr,
        is_starting_player: bool,
        info: SharedPlayerInfo,
    ) -> Self {
        let mut info_lock = info.lock().unwrap();
        if info_lock.username.is_empty() {
            info_lock.username = format!("Anon {}", Ulid::generate());
        }
        drop(info_lock);

        Self {
            game_id,
            peer,
            is_starting_player,
            info,
            mana: 0,
            empty_mana: 0,
            max_mana: 10,
            deck: CardCollection::default(),
            hand: CardCollection::default(),
            board: CardCollection::default(),
        }
    }

    pub fn setup_deck_from_deckcode(
        game: &SharedGame,
        starting_player: bool,
    ) -> Result<(), Box<dyn Error>> {
        let game_lock = game.lock().unwrap();
        let player = if game_lock.player1.lock().unwrap().is_starting_player == starting_player {
            game_lock.player1.clone()
        } else {
            game_lock.player2.clone()
        };
        drop(game_lock);

        let player_lock = player.lock().unwrap();
        let info_clone = player_lock.info.clone();
        let info_lock = info_clone.lock().unwrap();
        drop(player_lock);
        let deck = CardCollection::from_deckcode(game, info_lock.deckcode.as_str())?;
        deck.set_owner(Some(&player));
        drop(info_lock);
        let mut player_lock = player.lock().unwrap();
        player_lock.deck = deck;
        drop(player_lock);
        Ok(())
    }

    pub fn get_opponent(&self, game: &Game) -> SharedPlayer {
        if self.is_starting_player {
            game.player2.clone()
        } else {
            game.player1.clone()
        }
    }

    pub const fn get_opponent_peer(&self, game: &Game) -> SocketAddr {
        if self.is_starting_player {
            game.player2_peer
        } else {
            game.player1_peer
        }
    }

    pub fn draw_cards(&mut self, game: &Game, amount: u32) -> CardCollection {
        let mut cards = CardCollection::default();

        for _ in 0..amount {
            match self.deck.pop() {
                None => {
                    // No cards remaining in deck.
                    // TODO: Apply fatigue.
                }
                Some(card) => {
                    self.hand.push(card.clone());
                    cards.push(card);
                }
            }
        }

        // Send cards to client.
        let packet_data_cards: Vec<Card> = cards
            .clone()
            .into_iter()
            .map(|card| {
                let card_lock = card.lock().unwrap();
                let hash = card_lock.hash;

                let mut clone = card_lock.clone();
                drop(card_lock);
                clone.hash = hash;
                clone
            })
            .collect();

        let mut packet_data = Map::new();
        packet_data.insert(
            "cards".into(),
            serde_json::to_value(packet_data_cards).unwrap(),
        );
        game.send_packet_to(self.peer, Packet::new("draw_cards".into(), packet_data));

        // Opponent
        let mut packet_data = Map::new();
        packet_data.insert("amount".into(), cards.len().into());
        game.send_packet_to(
            self.get_opponent_peer(game),
            Packet::new("opponent_draw_cards".into(), packet_data),
        );

        cards
    }

    pub fn remove_card_from_location(&mut self, card: &SharedCard) {
        let card_lock = card.lock().unwrap();
        self.hand.erase_with_hash(card_lock.hash);
        self.deck.erase_with_hash(card_lock.hash);
        self.board.erase_with_hash(card_lock.hash);
    }

    pub fn play_card(&mut self, hash: Ulid) -> Result<(), PlayCardError> {
        let card = self
            .hand
            .get_with_hash(hash)
            .ok_or_else(|| {
                PlayCardError(format!(
                    "The card with hash '{hash}' is not in the player's hand."
                ))
            })?
            .clone();

        self.remove_card_from_location(&card);

        self.board.push(card);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlayCardError(String);

impl Display for PlayCardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for PlayCardError {}
