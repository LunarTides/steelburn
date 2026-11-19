use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::Parser;
use env_logger::Env;

use crate::{netcode::net::listen, pack::PackCollection};

mod card;
mod card_collection;
mod game;
mod lua;
mod netcode;
mod pack;
mod player;

type Shared<T> = Arc<Mutex<T>>;

#[derive(Parser, Clone)]
#[command(version, about = "A centralized server for steelburn", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    #[arg(long, help = "Enable cheats")]
    cheats: bool,

    // TODO: Actually implement config.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let cli = Cli::parse();
    let packs = PackCollection::discover("../packs".into())?;

    listen(cli, packs)
}
