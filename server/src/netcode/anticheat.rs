use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crate::{
    game::Game,
    netcode::net::{Packet, PeerMap},
};

pub const fn check(
    game: &Arc<Mutex<Game>>,
    peer_map: &PeerMap,
    peer: SocketAddr,
    packet: &Packet,
) -> bool {
    // TODO: Implement.
    true
}
