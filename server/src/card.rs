use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::player::SharedPlayer;

pub type SharedCard = Arc<Mutex<Card>>;

#[derive(Debug)]
pub struct Card {
    pub owner: SharedPlayer,
    pub name: String,
}

impl Display for Card {
    #[allow(clippy::significant_drop_tightening)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let owner_lock = self.owner.lock().unwrap();
        let info_lock = owner_lock.info.lock().unwrap();
        write!(f, "{} belonging to {}", self.name, info_lock.username)
    }
}
