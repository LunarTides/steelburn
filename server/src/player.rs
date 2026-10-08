use std::net::SocketAddr;

use ulid::Ulid;

use crate::netcode::net::SharedPlayerInfo;

#[derive(Debug)]
pub struct Player {
    pub game_id: Ulid,
    pub peer: SocketAddr,
    #[allow(
        clippy::struct_field_names,
        reason = "Without the 'player' at the end, it would be a bit unclear what this field is used for."
    )]
    pub is_starting_player: bool,
    pub info: SharedPlayerInfo,
}

impl Player {
    pub fn new(
        game_id: Ulid,
        peer: SocketAddr,
        is_starting_player: bool,
        info: SharedPlayerInfo,
    ) -> Self {
        let mut info_lock = info.lock().unwrap();
        if info_lock.username.is_empty() {
            info_lock.username = format!("Anon: {}", Ulid::generate());
        }
        drop(info_lock);

        Self {
            game_id,
            peer,
            is_starting_player,
            info,
        }
    }
}
