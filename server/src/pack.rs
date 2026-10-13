use std::{collections::HashMap, error::Error, ffi::OsStr, fs, path::PathBuf, sync::Mutex};

use serde::Deserialize;

use crate::{
    card::{Card, SharedCard},
    card_collection::CardCollection,
    game::SharedGame,
};

#[derive(Deserialize)]
pub struct Pack {
    #[serde(skip)]
    pub path: String,

    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub api_version: String,
    pub license: String,

    pub links: HashMap<String, String>,
    // pub requires: HashMap<String, PackRequire>,
    #[serde(skip)]
    pub cards: CardCollection,
}

impl Pack {
    pub fn discover(path: String) -> Vec<PathBuf> {
        let Ok(author_entries) = fs::read_dir(path) else {
            return Vec::new();
        };

        let mut paths = Vec::new();

        for author_entry in author_entries {
            let Ok(author_entry) = author_entry else {
                continue;
            };

            let Ok(entry_type) = author_entry.file_type() else {
                continue;
            };

            let entry_path = author_entry.path();
            if entry_type.is_dir() {
                // Discovered another directory. Check recursively check it.
                let Ok(entry_path_string) = entry_path.into_string() else {
                    continue;
                };

                paths.append(&mut Self::discover(entry_path_string));
                continue;
            }

            // Entry is probably a file.
            if entry_path.file_name() == Some(OsStr::new("pack.toml")) {
                paths.push(entry_path.parent().unwrap().canonicalize().unwrap());
            }
        }

        paths
    }

    pub fn discover_cards(pack_path: &str) -> Vec<PathBuf> {
        let Ok(card_entries) = fs::read_dir(PathBuf::from(&pack_path).join("cards")) else {
            return Vec::new();
        };

        let mut paths = Vec::new();

        for card_entry in card_entries {
            let Ok(card_entry) = card_entry else {
                continue;
            };

            let Ok(entry_type) = card_entry.file_type() else {
                continue;
            };

            let entry_path = card_entry.path();
            if entry_type.is_dir() {
                // Discovered another directory. Check recursively check it.
                let Some(entry_path_string) = entry_path.to_str() else {
                    continue;
                };

                paths.append(&mut Self::discover_cards(entry_path_string));
                continue;
            }

            // Entry is probably a file.
            paths.push(entry_path.canonicalize().unwrap());
        }

        paths
    }

    pub fn import(game: &SharedGame, path: &str) -> Result<Self, Box<dyn Error>> {
        let pack_file_contents = fs::read_to_string(PathBuf::from(path).join("pack.toml")).unwrap();
        let mut pack = toml::from_str::<Self>(&pack_file_contents)?;

        let mut cards = CardCollection::default();
        let card_paths = Self::discover_cards(path);

        for card_path in card_paths {
            let file_content = fs::read_to_string(card_path)?;

            let card = Card::parse_from_file_content(game, file_content)?;
            cards.push(SharedCard::new(Mutex::new(card)));
        }

        pack.cards = cards;
        Ok(pack)
    }
}
