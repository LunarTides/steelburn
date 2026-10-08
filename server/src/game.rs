use ulid::Ulid;

use crate::player::Player;

#[derive(Debug)]
pub struct Game {
    pub id: Ulid,
    pub player1: Player,
    pub player2: Player,
}

impl Game {
    pub fn new(player1_name: String, player2_name: String) -> Self {
        let id = Ulid::generate();
        Self {
            player1: Player::new(id.clone(), true, player1_name),
            player2: Player::new(id.clone(), false, player2_name),
            id,
        }
    }
}
