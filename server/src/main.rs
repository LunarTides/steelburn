use std::{error::Error, path::PathBuf};

use clap::Parser;
use env_logger::Env;

use crate::netcode::net::listen;

mod card;
mod card_collection;
mod game;
mod lua;
mod netcode;
mod pack;
mod player;

#[derive(Parser)]
#[command(version, about = "A centralized server for steelburn", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    // TODO: Actually implement config.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let cli = Cli::parse();

    let port = cli.port.unwrap_or(56040);
    listen(port)
}
