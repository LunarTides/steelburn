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

        // TODO: Actually get file content from cards.
        let file_content = r#"
            local card = {}
            card.name = "Sheep"
            card.cost = 1

            function card.onplay()
                print("[Lua/Sheep/onplay] Play")
                print("[Lua/Sheep/onplay] Is owner starting player: " .. tostring(owner.is_starting_player))
                owner.username = owner.username .. " (Lua)"
                print("[Lua/Sheep/onplay] Changed owner's name to: " .. owner.username)
            end

            return card
        "#;

        // TODO: Handle lua runtime errors.
        let sheep = Card::parse_from_file_content(game, file_content.into()).unwrap();

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
