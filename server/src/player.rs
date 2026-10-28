use std::{collections::HashMap, error::Error, fmt::Display, net::SocketAddr};

use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;

use crate::{
    Shared,
    card::{Card, Location, SharedCard},
    card_collection::CardCollection,
    game::{Game, SharedGame},
    netcode::{
        net::{Poisoned, SharedPlayerInfo},
        packet::ResponsePacket,
    },
};

pub type SharedPlayer = Shared<Player>;

#[derive(Debug, Clone, Serialize)]
pub struct Player {
    // TODO: Replace with a SharedGame reference.
    #[serde(skip)]
    pub game_id: Ulid,
    #[serde(skip)]
    pub peer: SocketAddr,
    #[allow(
        clippy::struct_field_names,
        reason = "Without the 'player' at the end, it would be a bit unclear what this field is used for."
    )]
    pub is_starting_player: bool,
    #[serde(skip)]
    pub info: SharedPlayerInfo,
    pub mana: i32,
    pub empty_mana: i32,
    pub max_mana: i32,
    #[serde(skip)]
    pub deck: CardCollection,
    #[serde(skip)]
    pub hand: CardCollection,
    #[serde(skip)]
    pub board: CardCollection,
    pub changes: HashMap<String, Value>,
}

impl Player {
    pub fn new(
        game_id: Ulid,
        peer: SocketAddr,
        is_starting_player: bool,
        info: SharedPlayerInfo,
    ) -> Self {
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
            changes: HashMap::new(),
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
        for card in &deck {
            let mut card_lock = card.lock().unwrap();
            // NOTE: We don't use `set_location` here since we don't want it to be stored in the card's changes.
            card_lock.location = Location::Deck;
        }

        drop(info_lock);
        let mut player_lock = player.lock().unwrap();
        player_lock.deck = deck;
        drop(player_lock);
        Ok(())
    }

    pub fn get_indexed_name(&self) -> String {
        format!("Player {}", if self.is_starting_player { 1 } else { 2 })
    }

    pub fn set_username(&mut self, username: String) {
        self.info.lock().unwrap().username.clone_from(&username);
        self.changes.insert("username".into(), username.into());
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

    pub fn draw_cards(
        &mut self,
        game: &Game,
        amount: u32,
        response: &mut Option<&mut ResponsePacket>,
    ) -> CardCollection {
        let mut cards = CardCollection::default();

        for _ in 0..amount {
            match self.deck.pop() {
                None => {
                    // No cards remaining in deck.
                    // TODO: Apply fatigue.
                }
                Some(card) => {
                    self.hand.push(card.clone());
                    // NOTE: We don't use `set_location` here since we don't want it to be stored in the card's changes.
                    // NOTE: ^ This is because the client doesn't know anything about this card yet, so any changes to it are irrelevant.
                    card.lock().unwrap().location = Location::Hand;
                    // TODO: Add changes to the card.
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

        if let Some(response) = response {
            let mut friendly_response = response.clone();
            friendly_response.packet_type = "draw_cards".into();
            friendly_response.response_data.insert(
                "cards".into(),
                serde_json::to_value(packet_data_cards).unwrap(),
            );
            friendly_response.is_starting_player = self.is_starting_player;
            friendly_response.use_player_changes(self);
            friendly_response.use_player_changes_shared(&self.get_opponent(game));
            game.send_packet_to(self.peer, friendly_response);

            // Opponent
            let mut opponent_response = response.clone();
            opponent_response.packet_type = "opponent_draw_cards".into();
            opponent_response
                .response_data
                .insert("amount".into(), cards.len().into());
            opponent_response.is_starting_player = self.is_starting_player;
            opponent_response.use_player_changes(self);
            opponent_response.use_player_changes_shared(&self.get_opponent(game));
            game.send_packet_to(self.get_opponent_peer(game), opponent_response);
        }

        cards
    }

    pub fn remove_card_from_location(&mut self, card: &SharedCard) {
        let card_lock = card.lock().unwrap();
        self.hand.erase_with_hash(card_lock.hash);
        self.deck.erase_with_hash(card_lock.hash);
        self.board.erase_with_hash(card_lock.hash);
    }

    pub fn play_card(&mut self, hash: Ulid) -> Result<SharedCard, PlayCardError> {
        let card = self
            .hand
            .get_with_hash(hash)
            .ok_or_else(|| {
                PlayCardError(format!(
                    "The card with hash '{hash}' is not in the player's hand."
                ))
            })?
            .clone();

        // TODO: Is unwrap safe here?
        if !self.can_afford(&card).unwrap() {
            return Err(PlayCardError("Can't afford card.".into()));
        }

        // TODO: Is unwrap safe here?
        self.reduce_mana(card.lock().unwrap().cost);
        self.remove_card_from_location(&card);

        self.board.push(card.clone());
        card.lock().unwrap().set_location(Location::Board);
        Ok(card)
    }

    pub fn set_mana(&mut self, mana: i32) {
        self.mana = mana;
        self.changes.insert("mana".into(), self.mana.into());
    }
    pub fn add_mana(&mut self, mana: i32) {
        self.mana = self.mana.saturating_add(mana);
        self.changes.insert("mana".into(), self.mana.into());
    }
    pub fn reduce_mana(&mut self, mana: i32) {
        self.mana = self.mana.saturating_sub(mana);
        self.changes.insert("mana".into(), self.mana.into());
    }

    pub fn can_afford<'a>(&self, card: &'a SharedCard) -> Result<bool, Poisoned<'a, Card>> {
        Ok(card.lock()?.cost <= self.mana)
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
