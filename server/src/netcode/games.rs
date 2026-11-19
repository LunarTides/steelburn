use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use log::{Level, log};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use ulid::Ulid;

use crate::{
    Cli, Shared,
    card_collection::CardCollection,
    game::{Game, SharedGame},
    netcode::{
        anticheat,
        net::{SharedPlayerInfo, Tx},
        packet::{Packet, ResponsePacket},
        peer_map::PeerMap,
    },
    pack::Pack,
};

type GameData = (SharedGame, SocketAddr, SocketAddr);

#[derive(Default, Clone, Debug)]
pub struct Games {
    data: Shared<Vec<GameData>>,
}

impl Games {
    pub fn add(
        &self,
        peer_map: &PeerMap,
        player1_peer: SocketAddr,
        player1_sink: Tx,
        player1_info: SharedPlayerInfo,
        player2_peer: SocketAddr,
        player2_sink: Tx,
        player2_info: SharedPlayerInfo,
    ) -> SharedGame {
        // TODO: Randomize who is player 1 and who is player 2.
        let game = Game::new(
            peer_map.clone(),
            player1_peer,
            player1_sink,
            player1_info,
            player2_peer,
            player2_sink,
            player2_info,
        );
        let shared_game = Arc::new(Mutex::new(game));
        self.data
            .lock()
            .unwrap()
            .push((shared_game.clone(), player1_peer, player2_peer));
        shared_game
    }

    pub fn remove(&self, peers: (SocketAddr, SocketAddr)) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        #[allow(clippy::suspicious_operation_groupings)]
        let queue_index = data
            .iter()
            .position(|x| x.1 == peers.0 || x.1 == peers.1 || x.2 == peers.0 || x.2 == peers.1)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn remove_by_peer(&self, peer: SocketAddr) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        let queue_index = data.iter().position(|x| x.1 == peer || x.2 == peer)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn is_poisoned(&self) -> bool {
        self.data.is_poisoned()
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` here."
    )]
    pub fn get_opposing_peer(&self, peer: SocketAddr) -> Option<SocketAddr> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        let opposing_peer = if peers.1 == peer { peers.2 } else { peers.1 };
        Some(opposing_peer)
    }

    /// Send a message to all peers connected to the same game as `peer`.
    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` nor `map_peers` here."
    )]
    pub fn send_message_by_peer(
        &self,
        peer_map: &PeerMap,
        peer: SocketAddr,
        message: Message,
    ) -> Option<()> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;

        // TODO: Is unwrap safe here?
        let map_peers = peer_map.get().unwrap();
        let sinks = map_peers
            .iter()
            .filter(|(addr, _)| *addr == &peers.1 || *addr == &peers.2)
            .map(|(_, (_, sink))| sink);

        for sink in sinks {
            sink.unbounded_send(message.clone()).unwrap();
        }

        Some(())
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "We can't drop `game_peers` here."
    )]
    pub fn get_game_from_peer(&self, peer: SocketAddr) -> Option<SharedGame> {
        let game_peers = self.data.lock().unwrap();
        let peers = game_peers.iter().find(|x| x.1 == peer || x.2 == peer)?;
        Some(peers.0.clone())
    }

    pub fn peer_in_game(&self, peer: SocketAddr) -> bool {
        let game_peers = self.data.lock().unwrap();
        game_peers.iter().any(|x| x.1 == peer || x.2 == peer)
    }

    // TODO: Return a result.
    // TODO: Split into multiple functions.
    pub fn handle_client_packet(
        &self,
        cli: &Cli,
        peer_map: &PeerMap,
        peer: SocketAddr,
        mut response: ResponsePacket,
    ) {
        let Some(game) = self.get_game_from_peer(peer) else {
            log!(Level::Error, "Could not get game from peer.");
            return;
        };

        if !anticheat::check(&game, peer_map, peer, &response) {
            return;
        }

        if response.packet_type == "verify_identity"
            || response.packet_type == "set_intention"
            || response.packet_type == "clear_intentions"
        {
            // Redirect to the opponent.
            let game_lock = game.lock().unwrap();
            let player = game_lock.get_player_from_peer(peer).unwrap();
            let player_lock = player.lock().unwrap();

            let opponent = player_lock.get_opponent();
            drop(player_lock);
            let opponent_lock = opponent.lock().unwrap();

            // Turn into a regular packet so we don't include unneccessary information.
            let packet: Packet = response.into();
            opponent_lock.sink.unbounded_send(packet.into()).unwrap();
            return;
        }
        if response.packet_type == "chat_message" {
            // Repeat to both players.
            let game_lock = game.lock().unwrap();
            response.response_data.clone_from(&response.request_data);
            response.is_starting_player = game_lock.player1_peer == peer;
            game_lock.send_packet(response);
            return;
        }

        if response.packet_type == "end_turn" {
            let mut game_lock = game.lock().unwrap();

            // Check if it's this player's turn.
            let is_player_turn = game_lock.current_player_peer == peer;
            if !is_player_turn {
                response.error = Some("It's not your turn.".into());

                let player = game_lock.get_player_from_peer(peer).unwrap();
                let mut player_lock = player.lock().unwrap();
                response.is_starting_player = player_lock.is_starting_player;
                // TODO: Is unwrap safe here?
                response.send_to(&mut player_lock).unwrap();
                return;
            }
            log!(
                Level::Debug,
                "Ending turn {} ({})",
                game_lock.turn,
                game_lock
                    .get_current_player()
                    .lock()
                    .unwrap()
                    .info
                    .lock()
                    .unwrap()
                    .username
            );

            // Repeat packet to both clients.
            game_lock.end_turn(&mut Some(&mut response));
            game_lock.send_packet(response);
            return;
        }
        if response.packet_type == "play_card" {
            let Some(hash_value) = response.request_data.get("hash") else {
                log!(
                    Level::Error,
                    "'play_card' packet doesn't have a 'hash' data field."
                );
                return;
            };
            let Some(hash_string) = hash_value.as_str() else {
                log!(
                    Level::Error,
                    "'play_card' packet 'hash' data field is not a string."
                );
                return;
            };
            let hash_ulid = match Ulid::from_string(hash_string) {
                Ok(hash_field) => hash_field,
                Err(err) => {
                    log!(
                        Level::Error,
                        "'play_card' packet 'hash' data field is not proper Ulid. {err}"
                    );
                    return;
                }
            };
            log!(Level::Debug, "Playing '{hash_value}'.");

            let game_lock = game.lock().unwrap();

            let is_player_turn = game_lock.current_player_peer == peer;
            if !is_player_turn {
                response.error = Some("It's not your turn.".into());

                let player = game_lock.get_player_from_peer(peer).unwrap();
                let mut player_lock = player.lock().unwrap();
                response.is_starting_player = player_lock.is_starting_player;
                // TODO: Is unwrap safe here?
                response.send_to(&mut player_lock).unwrap();
                return;
            }

            let player = game_lock.get_player_from_peer(peer).unwrap();
            let mut player_lock = player.lock().unwrap();

            let card = match player_lock.play_card(hash_ulid) {
                Err(err) => {
                    response.error = Some(err.to_string());
                    response.is_starting_player = player_lock.is_starting_player;
                    // TODO: Is unwrap safe here?
                    response.send_to(&mut player_lock).unwrap();
                    return;
                }
                Ok(card) => card,
            };

            log!(
                Level::Debug,
                "Board for {}: {:?} ({})",
                player_lock.peer,
                player_lock.board,
                player_lock.board.len()
            );
            drop(player_lock);

            let card_lock = card.lock().unwrap();

            // Run the `onplay` function on the lua side.
            if let Some(onplay) = &card_lock.onplay {
                game_lock
                    .lua_vm
                    .prepare_for_card(card.clone().into(), player.clone().into())
                    .unwrap();

                let onplay = onplay.clone();
                drop(card_lock);

                // onplay
                //     .set_environment(game_lock.lua_vm.lua.globals())
                //     .unwrap();

                // FIXME: If you set player's mana using cheats, then play a coin created before setting the mana, the coin will use the old mana
                if let Err(err) = onplay.call::<()>(()) {
                    response.error = Some(err.to_string());

                    let mut player_lock = player.lock().unwrap();
                    response.is_starting_player = player_lock.is_starting_player;
                    response.add_card("card", card);
                    // TODO: Is unwrap safe here?
                    response.send_to(&mut player_lock).unwrap();
                    return;
                }
            } else {
                drop(card_lock);
            }

            let player_lock = player.lock().unwrap();
            // Repeat packet to both clients.
            response.is_starting_player = player_lock.is_starting_player;
            drop(player_lock);

            response.add_card("card", card);
            game_lock.send_packet(response);
            return;
        }

        // Cheats
        if cli.cheats {
            if response.packet_type == "cheat_get_packs" {
                let pack_paths = Pack::discover("../packs".into());

                let game_lock = game.lock().unwrap();
                let mut packs = Vec::new();
                for pack_path in pack_paths {
                    let mut pack = Pack::import(&pack_path.to_string_lossy()).unwrap();
                    pack.import_cards(&game_lock).unwrap();
                    packs.push(pack);
                }

                response.response_data.insert("packs".into(), json!(packs));

                let player = game_lock.get_player_from_peer(peer).unwrap();
                let mut player_lock = player.lock().unwrap();
                response.is_starting_player = player_lock.is_starting_player;
                // TODO: Is unwrap safe here?
                response.send_to(&mut player_lock).unwrap();
                return;
            }

            if response.packet_type == "cheat_new_card" {
                let Some(author) = response.request_data.get("author") else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet doesn't have an 'author' data field."
                    );
                    return;
                };
                let Some(author) = author.as_str() else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet 'author' data field is not a string."
                    );
                    return;
                };

                let Some(pack_name) = response.request_data.get("pack") else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet doesn't have a 'pack' data field."
                    );
                    return;
                };
                let Some(pack_name) = pack_name.as_str() else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet 'pack' data field is not a string."
                    );
                    return;
                };

                let Some(card_id) = response.request_data.get("id") else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet doesn't have an 'id' data field."
                    );
                    return;
                };
                let Some(card_id) = card_id.as_str() else {
                    log!(
                        Level::Error,
                        "'cheat_new_card' packet 'id' data field is not a string."
                    );
                    return;
                };

                let game_lock = game.lock().unwrap();
                // TODO: Don't import locally.
                let mut pack = match Pack::import(&format!("../packs/@{author}/{pack_name}")) {
                    Ok(pack) => pack,
                    Err(err) => {
                        let mut err_msg = err.to_string();
                        if let Ok(err) = err.downcast::<io::Error>() {
                            err_msg = match err.kind() {
                                io::ErrorKind::NotFound => "No such pack exists.".into(),
                                _ => err_msg,
                            };
                        }

                        response.error = Some(err_msg);

                        let player = game_lock.get_player_from_peer(peer).unwrap();
                        let mut player_lock = player.lock().unwrap();
                        response.is_starting_player = player_lock.is_starting_player;
                        // TODO: Is unwrap safe here?
                        response.send_to(&mut player_lock).unwrap();
                        return;
                    }
                };
                if let Err(err) = pack.import_cards(&game_lock) {
                    response.error = Some(err.to_string());

                    let player = game_lock.get_player_from_peer(peer).unwrap();
                    let mut player_lock = player.lock().unwrap();
                    response.is_starting_player = player_lock.is_starting_player;
                    // TODO: Is unwrap safe here?
                    response.send_to(&mut player_lock).unwrap();
                    return;
                }
                let card = match pack.cards.get_with_id(card_id) {
                    Some(card) => card,
                    None => {
                        response.error = Some(format!("No card with id '{card_id}' in this pack."));

                        let player = game_lock.get_player_from_peer(peer).unwrap();
                        let mut player_lock = player.lock().unwrap();
                        response.is_starting_player = player_lock.is_starting_player;
                        // TODO: Is unwrap safe here?
                        response.send_to(&mut player_lock).unwrap();
                        return;
                    }
                };

                card.lock().unwrap().randomize_hash();

                let player = game_lock.get_player_from_peer(peer).unwrap();
                let mut player_lock = player.lock().unwrap();

                player_lock.add_to_hand(card);
                response.send_new_cards_packet(
                    &mut player_lock,
                    &CardCollection::from_vec(vec![card.clone()]),
                );
                return;
            }

            if response.packet_type == "cheat_set_mana" {
                let Some(is_starting_player) = response.request_data.get("is_starting_player")
                else {
                    log!(
                        Level::Error,
                        "'cheat_set_mana' packet doesn't have an 'is_starting_player' data field."
                    );
                    return;
                };
                let Some(is_starting_player) = is_starting_player.as_bool() else {
                    log!(
                        Level::Error,
                        "'cheat_set_mana' packet 'is_starting_player' data field is not a boolean."
                    );
                    return;
                };

                let Some(mana) = response.request_data.get("mana") else {
                    log!(
                        Level::Error,
                        "'cheat_set_mana' packet doesn't have a 'mana' data field."
                    );
                    return;
                };
                let Some(mana) = mana.as_i64() else {
                    log!(
                        Level::Error,
                        "'cheat_set_mana' packet 'mana' data field is not a number."
                    );
                    return;
                };
                let Ok(mana) = mana.try_into() else {
                    log!(
                        Level::Error,
                        "'cheat_set_mana' packet 'mana' data field is not a valid number."
                    );
                    return;
                };

                let game_lock = game.lock().unwrap();
                let player = if is_starting_player {
                    game_lock.player1.clone()
                } else {
                    game_lock.player2.clone()
                };

                player.lock().unwrap().set_mana(mana);
                player.lock().unwrap().set_empty_mana(mana);
                game_lock.send_packet(response);
                return;
            }

            if response.packet_type == "cheat_end_turn" {
                let mut game_lock = game.lock().unwrap();
                // Repeat packet to both clients.
                response.packet_type = "end_turn".into();
                game_lock.end_turn(&mut Some(&mut response));
                game_lock.send_packet(response);
                return;
            }
        }

        // TODO: Handle unknown packets.
    }
}
