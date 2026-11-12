use std::{collections::HashMap, error::Error, fmt::Display, net::SocketAddr};

use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;

use crate::{
    Shared,
    card::{Card, Location, SharedCard, Type},
    card_collection::CardCollection,
    game::SharedGame,
    netcode::{
        net::{Poisoned, SharedPlayerInfo, Tx},
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
    #[serde(skip)]
    pub sink: Tx,
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
    #[serde(skip)]
    pub oblivion: CardCollection,
    pub changes: HashMap<String, Value>,
    #[serde(skip)]
    opponent: Option<SharedPlayer>,
    #[serde(skip)]
    opponent_peer: Option<SocketAddr>,
}

impl Player {
    /// Call `set_opponent` on the returned player.
    pub fn new(
        game_id: Ulid,
        peer: SocketAddr,
        sink: Tx,
        is_starting_player: bool,
        info: SharedPlayerInfo,
    ) -> Self {
        Self {
            game_id,
            peer,
            sink,
            is_starting_player,
            info,
            mana: 0,
            empty_mana: 0,
            max_mana: 10,
            deck: CardCollection::default(),
            hand: CardCollection::default(),
            board: CardCollection::default(),
            oblivion: CardCollection::default(),
            changes: HashMap::new(),
            opponent: None,
            opponent_peer: None,
        }
    }

    pub fn set_opponent(&mut self, opponent: SharedPlayer, opponent_peer: SocketAddr) {
        self.opponent = Some(opponent);
        self.opponent_peer = Some(opponent_peer);
    }

    pub fn get_opponent(&self) -> SharedPlayer {
        let opponent = self.opponent.as_ref().unwrap();
        opponent.clone()
    }

    pub const fn get_opponent_peer(&self) -> SocketAddr {
        self.opponent_peer.unwrap()
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

    pub fn draw_cards(
        &mut self,
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
                    cards.push(card);
                }
            }
        }

        if let Some(response) = response {
            // Send cards to clients.
            response.is_starting_player = self.is_starting_player;
            // TODO: Is this smart? Does this properly send correct information for both peers?
            response.prepare_for_send(self, &mut self.get_opponent().lock().unwrap());

            // Friendly
            let mut friendly_response = response.clone();
            friendly_response.packet_type = "draw_cards".into();
            friendly_response.add_cards("cards", cards.clone());
            friendly_response.send_to(self).unwrap();

            // Opponent
            let mut opponent_response = response.clone();
            opponent_response.packet_type = "opponent_draw_cards".into();
            opponent_response
                .response_data
                .insert("amount".into(), cards.len().into());
            opponent_response
                .send_to_with_opponent(&mut self.get_opponent().lock().unwrap(), self)
                .unwrap();
        }

        cards
    }

    pub fn remove_card_from_location(&mut self, card: &SharedCard) {
        let mut card_lock = card.lock().unwrap();
        self.hand.erase_with_hash(card_lock.hash);
        self.deck.erase_with_hash(card_lock.hash);
        self.board.erase_with_hash(card_lock.hash);
        card_lock.set_location(Location::Oblivion);

        drop(card_lock);
        // Keep track of the oblivionated card until the next packet is sent.
        self.oblivion.push(card.clone());
    }

    pub fn add_to_hand(&mut self, card: &SharedCard) {
        self.hand.push(card.clone());
        // TODO: Broadcast to client.
        // self.changes.insert("hand".into(), ...);
        card.lock().unwrap().set_location(Location::Hand);
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

        if card.lock().unwrap().card_type == Type::Minion {
            self.board.push(card.clone());
            card.lock().unwrap().set_location(Location::Board);
        }

        Ok(card)
    }

    pub fn set_mana(&mut self, mana: i32) {
        self.mana = mana;
        self.changes.insert("mana".into(), self.mana.into());
    }
    pub fn add_mana(&mut self, mana: i32) {
        self.mana = self.mana.saturating_add(mana);
        if self.mana > self.empty_mana {
            self.mana = self.empty_mana;
        }

        self.changes.insert("mana".into(), self.mana.into());
    }
    pub fn reduce_mana(&mut self, mana: i32) {
        self.mana = self.mana.saturating_sub(mana);
        self.changes.insert("mana".into(), self.mana.into());
    }

    pub fn set_empty_mana(&mut self, empty_mana: i32) {
        self.empty_mana = empty_mana;
        if self.empty_mana > self.max_mana {
            self.empty_mana = self.max_mana;
        }

        self.changes
            .insert("empty_mana".into(), self.empty_mana.into());
    }
    pub fn add_empty_mana(&mut self, empty_mana: i32) {
        self.empty_mana = self.empty_mana.saturating_add(empty_mana);
        if self.empty_mana > self.max_mana {
            self.empty_mana = self.max_mana;
        }

        self.changes
            .insert("empty_mana".into(), self.empty_mana.into());
    }
    pub fn reduce_empty_mana(&mut self, empty_mana: i32) {
        self.empty_mana = self.empty_mana.saturating_sub(empty_mana);
        self.changes
            .insert("empty_mana".into(), self.empty_mana.into());
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
