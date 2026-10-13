use std::error::Error;

use env_logger::Env;

use crate::netcode::net::listen;

mod card;
mod card_collection;
mod game;
mod lua;
mod netcode;
mod pack;
mod player;

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    // TODO: Make this an argument using clap.
    let port = 56040;
    listen(port)
}
