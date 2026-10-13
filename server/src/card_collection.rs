use std::error::Error;

use crate::{card::SharedCard, game::SharedGame, pack::Pack, player::SharedPlayer};

#[derive(Debug, Default, Clone)]
pub struct CardCollection(Vec<SharedCard>);

impl IntoIterator for CardCollection {
    type Item = SharedCard;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<SharedCard> for CardCollection {
    fn from_iter<I: IntoIterator<Item = SharedCard>>(iter: I) -> Self {
        let mut c = Self::new();

        for i in iter {
            c.push(i);
        }

        c
    }
}

impl CardCollection {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_deckcode(game: &SharedGame, deckcode: &str) -> Result<Self, Box<dyn Error>> {
        if deckcode == "debug" {
            let mut cards = vec![];

            // TODO: Don't unwrap pack imports.
            let pack = Pack::import(game, "../packs/@steelburn/epsilon")?;
            let sheep = pack.cards.get_with_id("Sheep").unwrap();

            for _ in 0..30 {
                let sheep = sheep.lock().unwrap().clone();
                cards.push(sheep.into());
            }

            return Ok(Self(cards));
        }

        unimplemented!();
    }

    pub fn push(&mut self, card: SharedCard) {
        self.0.push(card);
    }

    pub fn get_with_id(&self, id: &str) -> Option<&SharedCard> {
        self.0.iter().find(|card| card.lock().unwrap().id == id)
    }

    pub fn set_owner(&self, owner: Option<&SharedPlayer>) {
        for card in &self.0 {
            let mut card_lock = card.lock().unwrap();
            card_lock.owner = owner.cloned();
        }
    }
}
