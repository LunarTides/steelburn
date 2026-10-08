use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use ulid::Ulid;

use crate::{card_collection::CardCollection, netcode::net::SharedPlayerInfo};

pub type SharedPlayer = Arc<Mutex<Player>>;

#[derive(Debug)]
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

    pub fn setup_deck_from_deckcode(player: &SharedPlayer) {
        let player_clone = player.clone();
        let mut player_lock = player_clone.lock().unwrap();
        let info_lock = player_lock.info.lock().unwrap();
        let deck = CardCollection::from_deckcode(info_lock.deckcode.as_str(), player);
        drop(info_lock);
        player_lock.deck = deck;
    }
}
