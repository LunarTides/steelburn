use std::{error::Error, fmt::Display, net::SocketAddr};

use mlua::prelude::*;
use serde::Serialize;
use serde_json::Map;
use ulid::Ulid;

use crate::{
    Shared,
    card::{Card, SharedCard},
    card_collection::CardCollection,
    game::{Game, SharedGame},
    netcode::net::{Packet, Poisoned, SharedPlayerInfo},
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
}

#[derive(Clone)]
pub struct LuaPlayer {
    pub player: SharedPlayer,
}

// TODO: Use mlua macros instead.
impl LuaUserData for LuaPlayer {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("is_starting_player", |_, this| {
            Ok(this.player.lock().unwrap().is_starting_player)
        });

        fields.add_field_method_get("username", |_, this| {
            Ok(this
                .player
                .lock()
                .unwrap()
                .info
                .lock()
                .unwrap()
                .username
                .clone())
        });
        fields.add_field_method_set("username", |_, this, val| {
            this.player.lock().unwrap().info.lock().unwrap().username = val;
            Ok(())
        });

        fields.add_field_method_get("mana", |_, this| Ok(this.player.lock().unwrap().mana));
        fields.add_field_method_set("mana", |_, this, val| {
            this.player.lock().unwrap().mana = val;
            Ok(())
        });
    }
}

// impl FromLua for Player {
//     fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
//         match value {
//             LuaValue::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
//             _ => unreachable!(),
//         }
//     }
// }

impl From<SharedPlayer> for LuaPlayer {
    fn from(value: SharedPlayer) -> Self {
        LuaPlayer { player: value }
    }
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

    pub fn play_card(&mut self, game_lock: &Game, hash: Ulid) -> Result<SharedCard, PlayCardError> {
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
        Ok(card)
    }

    pub fn reduce_mana(&mut self, cost: i32) {
        self.mana -= cost;
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
