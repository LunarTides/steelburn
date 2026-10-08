use std::error::Error;

use env_logger::Env;

use crate::netcode::net::listen;

mod card;
mod card_collection;
mod game;
mod netcode;
mod player;

fn main() -> Result<(), Box<dyn Error>> {
    // let game = Game::new("En".to_string(), "To".to_string());
    // let sheep = Card {
    //     owner: &game.player1,
    //     name: "Sheep".into(),
    // };

    // println!("Hello, world!");
    // println!("{sheep}");

    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    // TODO: Make this an argument using clap.
    let port = 56040;
    listen(port)
}
