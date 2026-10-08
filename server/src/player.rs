use ulid::Ulid;

#[derive(Default, Debug)]
pub struct Player {
    pub game_id: Ulid,
    #[allow(
        clippy::struct_field_names,
        reason = "Without the 'player' at the end, it would be a bit unclear what this field is used for."
    )]
    pub is_starting_player: bool,
    pub name: String,
}

impl Player {
    pub const fn new(game_id: Ulid, is_starting_player: bool, name: String) -> Self {
        Self {
            game_id,
            is_starting_player,
            name,
        }
    }
}
