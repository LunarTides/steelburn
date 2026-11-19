use std::net::SocketAddr;

use log::{Level, log};
use serde_json::json;
use ulid::Ulid;

use crate::{
    Cli,
    card::SharedCard,
    card_collection::CardCollection,
    game::SharedGame,
    lua::{LuaCard, LuaPlayer},
    netcode::{
        anticheat,
        games::Games,
        packet::{Packet, ResponsePacket},
        peer_map::PeerMap,
    },
    pack::PackCollection,
    player::SharedPlayer,
};

struct Info<'a> {
    cli: &'a Cli,
    peer_map: &'a PeerMap,
    games: &'a Games,
    game: SharedGame,
    peer: SocketAddr,
}

// TODO: Return a result.
pub fn handle_packet(
    cli: &Cli,
    peer_map: &PeerMap,
    games: &Games,
    peer: SocketAddr,
    response: ResponsePacket,
) {
    let Some(game) = games.get_game_from_peer(peer) else {
        log!(Level::Error, "Could not get game from peer.");
        return;
    };

    if !anticheat::check(&game, peer_map, peer, &response) {
        return;
    }

    let info = Info {
        cli,
        peer_map,
        games,
        game,
        peer,
    };

    if response.packet_type == "verify_identity"
        || response.packet_type == "set_intention"
        || response.packet_type == "clear_intentions"
    {
        // Redirect to the opponent.
        send_to_opponent(&info, response);
        return;
    }
    if response.packet_type == "chat_message" {
        // Repeat to both players.
        send_to_players(&info, response);
        return;
    }

    if response.packet_type == "end_turn" {
        end_turn(&info, response);
        return;
    }
    if response.packet_type == "play_card" {
        play_card(&info, response);
        return;
    }
    if response.packet_type == "attack" {
        attack(&info, response);
        return;
    }

    // Cheats
    if cli.cheats {
        if response.packet_type == "cheat_get_packs" {
            cheat_get_packs(&info, response);
            return;
        }
        if response.packet_type == "cheat_new_card" {
            cheat_new_card(&info, response);
            return;
        }
        if response.packet_type == "cheat_set_mana" {
            cheat_set_mana(&info, response);
            return;
        }
        if response.packet_type == "cheat_end_turn" {
            cheat_end_turn(&info, response);
            return;
        }
    }

    // TODO: Handle unknown packets.
}

fn send_to_players(info: &Info, mut response: ResponsePacket) {
    // Repeat to both players.
    let game_lock = info.game.lock().unwrap();
    response.response_data.clone_from(&response.request_data);
    response.is_starting_player = game_lock.player1_peer == info.peer;
    game_lock.send_packet(response);
}

fn send_to_opponent(info: &Info, response: ResponsePacket) {
    // Redirect to the opponent.
    let game_lock = info.game.lock().unwrap();
    let player = game_lock.get_player_from_peer(info.peer).unwrap();
    let player_lock = player.lock().unwrap();

    let opponent = player_lock.get_opponent();
    drop(player_lock);
    let opponent_lock = opponent.lock().unwrap();

    // Turn into a regular packet so we don't include unneccessary information.
    let packet: Packet = response.into();
    opponent_lock.sink.unbounded_send(packet.into()).unwrap();
}

fn end_turn(info: &Info, mut response: ResponsePacket) {
    let mut game_lock = info.game.lock().unwrap();

    // Check if it's this player's turn.
    let is_player_turn = game_lock.current_player_peer == info.peer;
    if !is_player_turn {
        response.error = Some("It's not your turn.".into());

        let player = game_lock.get_player_from_peer(info.peer).unwrap();
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
}

fn play_card(info: &Info, mut response: ResponsePacket) {
    let Some(hash_value) = response.request_data.get("hash") else {
        response.error = Some("'play_card' packet doesn't have a 'hash' data field.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };
    let Some(hash_string) = hash_value.as_str() else {
        response.error = Some("'play_card' packet 'hash' data field is not a string.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };
    let hash_ulid = match Ulid::from_string(hash_string) {
        Ok(hash_field) => hash_field,
        Err(err) => {
            response.error = Some(format!(
                "'play_card' packet 'hash' data field is not a proper Ulid. {err}"
            ));

            let game_lock = info.game.lock().unwrap();
            let player = game_lock.get_player_from_peer(info.peer).unwrap();
            let mut player_lock = player.lock().unwrap();
            response.is_starting_player = player_lock.is_starting_player;
            // TODO: Is unwrap safe here?
            response.send_to(&mut player_lock).unwrap();
            return;
        }
    };
    log!(Level::Debug, "Playing '{hash_value}'.");

    let game_lock = info.game.lock().unwrap();

    let is_player_turn = game_lock.current_player_peer == info.peer;
    if !is_player_turn {
        response.error = Some("It's not your turn.".into());

        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    }

    let player = game_lock.get_player_from_peer(info.peer).unwrap();
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
        let onplay = onplay.clone();
        drop(card_lock);

        // onplay
        //     .set_environment(game_lock.lua_vm.lua.globals())
        //     .unwrap();

        // FIXME: Local variables are shared between cards with the same id.
        if let Err(err) = onplay.call::<()>((
            <SharedCard as Into<LuaCard>>::into(card.clone()),
            <SharedPlayer as Into<LuaPlayer>>::into(player.clone()),
        )) {
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
}

fn attack(info: &Info, mut response: ResponsePacket) {
    // {
    //   "attacker": "D74732473892" | "Player 1"
    //   "target": "D5443267842DF" | "Player 2"
    // }

    let Some(attacker) = response.request_data.get("attacker") else {
        response.error = Some("'attack' packet doesn't have an 'attacker' data field.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };
    let Some(attacker) = attacker.as_str() else {
        response.error = Some("'attack' packet 'attacker' data field is not a string.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };

    let Some(target) = response.request_data.get("target") else {
        response.error = Some("'attack' packet doesn't have a 'target' data field.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };
    let Some(target) = target.as_str() else {
        response.error = Some("'attack' packet 'target' data field is not a string.".into());

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };

    let game_lock = info.game.lock().unwrap();
    let is_player_1 = game_lock.is_peer_starting_player(info.peer);
    drop(game_lock);

    let (attacker_card, attacker_player) =
        _get_player_or_card(info, response.clone(), is_player_1, attacker, "attacker");
    if attacker_card.is_none() && attacker_player.is_none() {
        return;
    }

    let (target_card, target_player) =
        _get_player_or_card(info, response.clone(), !is_player_1, target, "target");
    if target_card.is_none() && target_player.is_none() {
        return;
    }

    if let Some(attacker) = attacker_card {
        if let Some(target) = target_card {
            // TODO: Do error response if this fails.
            // FIXME: This doesn't send the changes to the opposing card.
            attacker
                .lock()
                .unwrap()
                .attack_card(&mut target.lock().unwrap());
        } else if let Some(target) = target_player {
            // TODO: Do error response if this fails.
            attacker
                .lock()
                .unwrap()
                .attack_player(&mut target.lock().unwrap());
        }
    } else if let Some(attacker) = attacker_player {
        if let Some(target) = target_card {
            // TODO: Do error response if this fails.
            attacker
                .lock()
                .unwrap()
                .attack_card(&mut target.lock().unwrap());
        } else if let Some(target) = target_player {
            // TODO: Do error response if this fails.
            attacker
                .lock()
                .unwrap()
                .attack_player(&mut target.lock().unwrap());
        }
    }

    let game_lock = info.game.lock().unwrap();
    let player = game_lock.get_player_from_peer(info.peer).unwrap();
    let player_lock = player.lock().unwrap();
    // Repeat packet to both clients.
    response.is_starting_player = player_lock.is_starting_player;
    drop(player_lock);

    game_lock.send_packet(response);
}

fn _get_player_or_card(
    info: &Info,
    mut response: ResponsePacket,
    player1: bool,
    string: &str,
    packet_key: &str,
) -> (Option<SharedCard>, Option<SharedPlayer>) {
    let mut card = None;
    let mut player = None;

    if let Ok(hash) = Ulid::from_string(string) {
        let game_lock = info.game.lock().unwrap();
        let player = if player1 {
            game_lock.player1.clone()
        } else {
            game_lock.player2.clone()
        };
        let mut player_lock = player.lock().unwrap();

        card = match player_lock.board.get_with_hash(hash) {
            Some(card) => Some(card),
            None => {
                response.error = Some(format!(
                    "'attack' packet '{}' data field was not found on the board.",
                    packet_key
                ));
                response.is_starting_player = player_lock.is_starting_player;
                // TODO: Is unwrap safe here?
                response.send_to(&mut player_lock).unwrap();
                return (None, None);
            }
        }
        .cloned();
    } else if string.starts_with("Player ") {
        let Some(player_index) = string.split("Player ").nth(1) else {
            response.error = Some(format!(
                "'attack' packet '{}' data field no such player '{}'.",
                packet_key, string
            ));

            let game_lock = info.game.lock().unwrap();
            let player = game_lock.get_player_from_peer(info.peer).unwrap();
            let mut player_lock = player.lock().unwrap();
            response.is_starting_player = player_lock.is_starting_player;
            // TODO: Is unwrap safe here?
            response.send_to(&mut player_lock).unwrap();
            return (None, None);
        };

        let game_lock = info.game.lock().unwrap();
        if player_index == "1" {
            player = Some(game_lock.player1.clone());
        } else if player_index == "2" {
            player = Some(game_lock.player2.clone());
        } else {
            response.error = Some(format!(
                "'attack' packet '{}' data field no such player '{}'.",
                packet_key, string
            ));

            let player = game_lock.get_player_from_peer(info.peer).unwrap();
            let mut player_lock = player.lock().unwrap();
            response.is_starting_player = player_lock.is_starting_player;
            // TODO: Is unwrap safe here?
            response.send_to(&mut player_lock).unwrap();
            return (None, None);
        }
    } else {
        response.error = Some(format!(
            "'attack' packet '{}' data field '{}' is not a valid target.",
            packet_key, string
        ));

        let game_lock = info.game.lock().unwrap();
        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return (None, None);
    }

    (card, player.clone())
}

fn cheat_get_packs(info: &Info, mut response: ResponsePacket) {
    let mut game_lock = info.game.lock().unwrap();
    game_lock.packs = match PackCollection::discover("../packs".into()) {
        Ok(packs) => packs,
        Err(err) => {
            response.error = Some(err.to_string());

            let player = game_lock.get_player_from_peer(info.peer).unwrap();
            let mut player_lock = player.lock().unwrap();
            response.is_starting_player = player_lock.is_starting_player;
            // TODO: Is unwrap safe here?
            response.send_to(&mut player_lock).unwrap();
            return;
        }
    };

    let packs = game_lock.packs.lock();
    response.response_data.insert("packs".into(), json!(packs));

    let player = game_lock.get_player_from_peer(info.peer).unwrap();
    let mut player_lock = player.lock().unwrap();
    response.is_starting_player = player_lock.is_starting_player;
    // TODO: Is unwrap safe here?
    response.send_to(&mut player_lock).unwrap();
}

fn cheat_new_card(info: &Info, mut response: ResponsePacket) {
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

    let game_lock = info.game.lock().unwrap();
    let Some(pack) = game_lock.packs.get_pack(author, pack_name) else {
        response.error = Some("No such pack exists.".into());

        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };
    let pack_lock = pack.lock().unwrap();
    let Some(card) = pack_lock.cards.get_with_id(card_id) else {
        response.error = Some(format!("No card with id '{card_id}' in this pack."));

        let player = game_lock.get_player_from_peer(info.peer).unwrap();
        let mut player_lock = player.lock().unwrap();
        response.is_starting_player = player_lock.is_starting_player;
        // TODO: Is unwrap safe here?
        response.send_to(&mut player_lock).unwrap();
        return;
    };

    let card = &card.lock().unwrap().clone().into();
    drop(pack_lock);

    let player = game_lock.get_player_from_peer(info.peer).unwrap();
    let mut player_lock = player.lock().unwrap();

    player_lock.add_to_hand(card);
    response.send_new_cards_packet(
        &mut player_lock,
        &CardCollection::from_vec(vec![card.clone()]),
    );
}

fn cheat_set_mana(info: &Info, response: ResponsePacket) {
    let Some(is_starting_player) = response.request_data.get("is_starting_player") else {
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

    let game_lock = info.game.lock().unwrap();
    let player = if is_starting_player {
        game_lock.player1.clone()
    } else {
        game_lock.player2.clone()
    };

    player.lock().unwrap().set_mana(mana);
    player.lock().unwrap().set_empty_mana(mana);
    game_lock.send_packet(response);
}

fn cheat_end_turn(info: &Info, mut response: ResponsePacket) {
    let mut game_lock = info.game.lock().unwrap();
    // Repeat packet to both clients.
    response.packet_type = "end_turn".into();
    game_lock.end_turn(&mut Some(&mut response));
    game_lock.send_packet(response);
}
