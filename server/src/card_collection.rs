use std::error::Error;

use ulid::Ulid;

use crate::{
    card::{Card, SharedCard},
    game::SharedGame,
    pack::Pack,
    player::SharedPlayer,
};

/// A list of cards.
#[derive(Debug, Default, Clone)]
pub struct CardCollection {
    cards: Vec<SharedCard>,
    hashes: Vec<Ulid>,
    dirty: bool,
}

impl IntoIterator for CardCollection {
    type Item = SharedCard;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.into_iter()
    }
}
impl IntoIterator for &CardCollection {
    type Item = SharedCard;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.cards.clone().into_iter()
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
    /// Creates a card collection from a vector of shared cards. This locks each card mutex in the list.
    pub fn from_vec(vec: Vec<SharedCard>) -> Self {
        let mut hashes = Vec::new();
        for card in &vec {
            hashes.push(card.lock().unwrap().hash);
        }

        Self {
            cards: vec,
            hashes,
            dirty: false,
        }
    }

    /// Creates a card collection from a deckcode.
    pub fn from_deckcode(game: &SharedGame, deckcode: &str) -> Result<Self, Box<dyn Error>> {
        if deckcode == "debug" {
            let mut cards = vec![];

            let pack = Pack::import(&game.lock().unwrap(), "../packs/@steelburn/epsilon")?;
            let sheep = pack.cards.get_with_id("Sheep").unwrap();

            for _ in 0..30 {
                let sheep = sheep.lock().unwrap().clone();
                cards.push(sheep.into());
            }

            return Ok(Self::from_vec(cards));
        }

        // TODO: Implement.
        unimplemented!();
    }

    /// Adds a card to the end of the list.
    /// This makes this card collection dirty.
    /// This locks the card mutex.
    pub fn push(&mut self, card: SharedCard) {
        self.hashes.push(card.lock().unwrap().hash);
        self.cards.push(card);
        self.make_dirty();
    }

    /// Removes and return the card at the end of the list.
    /// This makes this card collection dirty.
    pub fn pop(&mut self) -> Option<SharedCard> {
        self.make_dirty();
        self.hashes.pop();
        self.cards.pop()
    }

    /// Removes a card from the collection.
    /// This makes this card collection dirty.
    /// This locks the card mutex contained in the collection.
    pub fn erase(&mut self, val: &Card) -> Option<()> {
        let index = self
            .cards
            .iter()
            .position(|card| card.lock().unwrap().hash == val.hash)?;
        self.hashes.remove(index);
        self.cards.remove(index);
        self.make_dirty();
        Some(())
    }

    /// Removes the card from the collection that has the specified hash.
    /// This makes this card collection dirty.
    pub fn erase_with_hash(&mut self, hash: Ulid) -> Option<()> {
        let index = self.hashes.iter().position(|h| *h == hash)?;
        self.hashes.remove(index);
        self.cards.remove(index);
        self.make_dirty();
        Some(())
    }

    /// Removes all cards from the collection.
    /// This makes this card collection dirty.
    pub fn clear(&mut self) {
        self.hashes.clear();
        self.cards.clear();
        self.make_dirty();
    }

    /// Returns whether or not this card collection has changed since creation. The dirty flag can be unset by calling `clear_dirty()`.
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Sets the dirty flag on this card collection.
    pub const fn make_dirty(&mut self) {
        self.dirty = true;
    }

    /// Unsets the dirty flag on this card collection.
    pub const fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Returns the amount of cards in this collection.
    pub const fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns the first instance of the card with the specified id, if any.
    /// This locks all card mutexes in the collection.
    pub fn get_with_id(&self, id: &str) -> Option<&SharedCard> {
        self.cards.iter().find(|card| card.lock().unwrap().id == id)
    }

    /// Returns the card with the specified hash, if any.
    pub fn get_with_hash(&self, hash: Ulid) -> Option<&SharedCard> {
        let index = self.hashes.iter().position(|h| *h == hash)?;
        self.cards.get(index)
    }

    /// Sets and overrides the `owner` property of all cards in this collection.
    /// This locks all card mutexes in the collection, and locks the player mutex.
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
