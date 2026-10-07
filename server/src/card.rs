use std::fmt::Display;

use crate::player::Player;

pub struct Card<'a> {
    pub owner: &'a Player,
    pub name: String,
}

impl Display for Card<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} belonging to {}", self.name, self.owner.name)
    }
}
