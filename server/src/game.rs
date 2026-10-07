use crate::player::Player;

pub struct Game {
    pub id: String,
    pub player1: Player,
    pub player2: Player,
}

impl Game {
    pub fn new(id: String, player1_name: String, player2_name: String) -> Self {
        Self {
            player1: Player::new(id.clone(), true, player1_name),
            player2: Player::new(id.clone(), false, player2_name),
            id,
        }
    }
}
