use std::net::SocketAddr;

use crate::{
    game::SharedGame,
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
