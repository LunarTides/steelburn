use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use mlua::prelude::*;
use ulid::Ulid;

use crate::{card_collection::CardCollection, game::SharedGame, netcode::net::SharedPlayerInfo};

pub type SharedPlayer = Arc<Mutex<Player>>;

#[derive(Debug, Clone)]
pub struct Player {
    pub game_id: Ulid,
    pub peer: SocketAddr,
    #[allow(
        clippy::struct_field_names,
        reason = "Without the 'player' at the end, it would be a bit unclear what this field is used for."
    )]
    pub is_starting_player: bool,
    pub info: SharedPlayerInfo,
    pub deck: CardCollection,
}

impl LuaUserData for Player {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("is_starting_player", |_, this| Ok(this.is_starting_player));
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
            deck: CardCollection::default(),
        }
    }

    pub fn setup_deck_from_deckcode(game: &SharedGame, starting_player: bool) {
        let game_lock = game.lock().unwrap();
        let player = if game_lock.player1.lock().unwrap().is_starting_player == starting_player {
            game_lock.player1.clone()
        } else {
            game_lock.player2.clone()
        };
        drop(game_lock);

        let mut player_lock = player.lock().unwrap();
        let info_lock = player_lock.info.lock().unwrap();
        let deck = CardCollection::from_deckcode(game, info_lock.deckcode.as_str());
        drop(info_lock);
        player_lock.deck = deck;
    }
}
