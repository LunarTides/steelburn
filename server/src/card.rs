use std::fmt::Display;

use crate::player::Player;

pub struct Card<'a> {
    pub owner: &'a Player,
    pub name: String,
}

impl Display for Card<'_> {
    #[allow(clippy::significant_drop_tightening)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info_lock = self.owner.info.lock().unwrap();
        write!(f, "{} belonging to {}", self.name, info_lock.username)
    }
}
