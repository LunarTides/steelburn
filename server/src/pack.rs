use std::{collections::HashMap, sync::Mutex};

use crate::{
    card::{Card, SharedCard},
    card_collection::CardCollection,
    game::SharedGame,
};

pub struct Pack {
    pub path: String,

    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub api_version: String,
    pub license: String,

    pub links: HashMap<String, String>,
    // pub requires: HashMap<String, PackRequire>,
    pub cards: CardCollection,
}

impl Pack {
    pub fn discover() -> Vec<String> {
        todo!()
    }

    // pub fn import(path: String) -> Self {
    pub fn import_test(game: &SharedGame) -> Self {
        let mut cards = CardCollection::default();

        let file_content = r#"
            card.name = "Sheep"
            card.cost = 1

            function card.play()
                print("[Sheep/play] is player1 starting player: " .. player1.is_starting_player())
            end
        "#;

        let game_lock = game.lock().unwrap();
        let sheep = Card::parse_from_file_content(&game_lock.lua_vm, file_content.into()).unwrap();
        drop(game_lock);

        cards.push(SharedCard::new(Mutex::new(sheep)));

        // TODO: Actually import.
        Self {
            path: "../packs/@steelburn/epsilon".into(),
            name: "epsilon".into(),
            description: "The first official pack.".into(),
            author: "steelburn".into(),
            version: "0.1.0".into(),
            api_version: "0.1.0".into(),
            license: "GPL-3.0".into(),
            links: HashMap::from([(
                "repository".to_string(),
                "https://git.lunartides.dev/LunarTides/steelburn".to_string(),
            )]),
            cards,
        }
    }
}
