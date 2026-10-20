use std::error::Error;

use ulid::Ulid;

use crate::{
    card::{Card, SharedCard},
    game::SharedGame,
    pack::Pack,
    player::SharedPlayer,
};

#[derive(Debug, Default, Clone)]
pub struct CardCollection {
    cards: Vec<SharedCard>,
    hashes: Vec<Ulid>,
}

impl IntoIterator for CardCollection {
    type Item = SharedCard;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
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
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Use proper From thingy.
    pub fn from_vec(vec: Vec<SharedCard>) -> Self {
        let mut hashes = Vec::new();
        for card in &vec {
            hashes.push(card.lock().unwrap().hash);
        }

        Self { cards: vec, hashes }
    }

    pub fn from_deckcode(game: &SharedGame, deckcode: &str) -> Result<Self, Box<dyn Error>> {
        if deckcode == "debug" {
            let mut cards = vec![];

            let pack = Pack::import(game, "../packs/@steelburn/epsilon")?;
            let sheep = pack.cards.get_with_id("Sheep").unwrap();

            for _ in 0..30 {
                let sheep = sheep.lock().unwrap().clone();
                cards.push(sheep.into());
            }

            return Ok(Self::from_vec(cards));
        }

        unimplemented!();
    }

    pub fn push(&mut self, card: SharedCard) {
        self.hashes.push(card.lock().unwrap().hash);
        self.cards.push(card);
    }

    pub fn pop(&mut self) -> Option<SharedCard> {
        self.hashes.pop();
        self.cards.pop()
    }

    pub fn erase(&mut self, val: &Card) {
        let Some(index) = self
            .cards
            .iter()
            .position(|card| card.lock().unwrap().hash == val.hash)
        else {
            return;
        };
        self.hashes.remove(index);
        self.cards.remove(index);
    }

    pub fn erase_with_hash(&mut self, hash: Ulid) {
        let Some(index) = self.hashes.iter().position(|h| *h == hash) else {
            return;
        };
        self.hashes.remove(index);
        self.cards.remove(index);
    }

    pub const fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn get_with_id(&self, id: &str) -> Option<&SharedCard> {
        self.cards.iter().find(|card| card.lock().unwrap().id == id)
    }

    pub fn get_with_hash(&self, hash: Ulid) -> Option<&SharedCard> {
        let index = self.hashes.iter().position(|h| *h == hash)?;
        self.cards.get(index)
    }

    pub fn set_owner(&self, owner: Option<&SharedPlayer>) {
        for card in &self.cards {
            let mut card_lock = card.lock().unwrap();
            card_lock.owner = owner.cloned();

            if let Some(owner) = owner {
                card_lock.is_starting_player = owner.lock().unwrap().is_starting_player;
            }
        }
    }
}
