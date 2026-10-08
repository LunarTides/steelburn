use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use crate::{
    game::{Game, SharedGame},
    netcode::net::{Packet, PeerMap},
};

pub const fn check(
    game: &SharedGame,
    peer_map: &PeerMap,
    peer: SocketAddr,
    packet: &Packet,
) -> bool {
    // TODO: Implement.
    true
}
