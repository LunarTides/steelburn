use std::sync::Mutex;

use crate::{
    card::{Card, SharedCard},
    player::SharedPlayer,
};

#[derive(Debug, Default)]
pub struct CardCollection(Vec<SharedCard>);

impl CardCollection {
    pub fn from_deckcode(deckcode: &str, owner: &SharedPlayer) -> Self {
        if deckcode == "debug" {
            let mut cards = vec![];
            for _ in 0..30 {
                let sheep = Card {
                    owner: owner.clone(),
                    name: "Sheep".into(),
                };
                cards.push(SharedCard::new(Mutex::new(sheep)));
            }

            return Self(cards);
        }

        unimplemented!();
    }
}
