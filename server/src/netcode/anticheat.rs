use std::net::SocketAddr;

use crate::{
    game::SharedGame,
    netcode::{net::ResponsePacket, peer_map::PeerMap},
};

pub const fn check(
    game: &SharedGame,
    peer_map: &PeerMap,
    peer: SocketAddr,
    packet: &ResponsePacket,
) -> bool {
    // TODO: Implement.
    true
}
