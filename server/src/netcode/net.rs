use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::{StreamExt, TryStreamExt, future, pin_mut};
use log::{Level, log};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use ulid::Ulid;

use crate::{
    Shared,
    card::SharedCard,
    netcode::{games::Games, peer_map::PeerMap, queue::Queue},
    player::{Player, SharedPlayer},
};

pub type Tx = UnboundedSender<Message>;
pub type SharedPlayerInfo = Shared<PlayerInfo>;
pub type Poisoned<'a, T> = PoisonError<MutexGuard<'a, T>>;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct PlayerInfo {
    pub username: String,
    // Don't send the deckcode to the clients.
    #[serde(skip_serializing)]
    pub deckcode: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    #[serde(rename = "type")]
    pub packet_type: String,
    pub id: String,
    pub data: HashMap<String, Value>,
}
impl Packet {
    pub const fn new(packet_type: String, id: String, data: HashMap<String, Value>) -> Self {
        Self {
            packet_type,
            id,
            data,
        }
    }
}
impl From<Packet> for Message {
    fn from(val: Packet) -> Self {
        Self::Text(serde_json::to_string(&val).unwrap().into())
    }
}
impl From<&Packet> for Message {
    fn from(val: &Packet) -> Self {
        Self::Text(serde_json::to_string(&val).unwrap().into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResponsePacket {
    #[serde(rename = "type")]
    pub packet_type: String,
    pub id: Option<String>,
    pub is_starting_player: bool,
    pub request_data: HashMap<String, Value>,
    pub response_data: HashMap<String, Value>,
    pub error: Option<String>,
    pub changes: HashMap<String, HashMap<String, Value>>,
}
impl ResponsePacket {
    pub fn new(
        packet_type: String,
        id: Option<String>,
        is_starting_player: bool,
        response_data: HashMap<String, Value>,
    ) -> Self {
        Self {
            packet_type,
            id,
            is_starting_player,
            request_data: HashMap::new(),
            response_data,
            error: None,
            changes: HashMap::new(),
        }
    }

    pub fn use_player_changes(&mut self, player: &mut Player) {
        if player.changes.is_empty() {
            return;
        }

        let name = player.get_indexed_name();

        if !self.changes.contains_key(&name) {
            self.changes.insert(name.clone(), HashMap::new());
        }

        let changes = self.changes.get_mut(&name).unwrap();
        for (k, v) in player.changes.clone() {
            changes.insert(k, v);
        }

        player.changes.clear();

        // Use card changes from player.
        // FIXME: Cards whose location is changed to Oblivion won't be used here.
        for card in &player.deck {
            self.use_card_changes_shared(&card);
        }
        for card in &player.hand {
            self.use_card_changes_shared(&card);
        }
        for card in &player.board {
            self.use_card_changes_shared(&card);
        }
    }

    pub fn use_player_changes_shared(&mut self, player: &SharedPlayer) {
        let mut player_lock = player.lock().unwrap();
        self.use_player_changes(&mut player_lock)
    }

    pub fn use_card_changes_shared(&mut self, card: &SharedCard) {
        // TODO: Don't send sensitive changes to the opposing peer.
        let mut card_lock = card.lock().unwrap();
        if card_lock.changes.is_empty() {
            return;
        }

        let hash = card_lock.hash.to_string();
        if !self.changes.contains_key(&hash) {
            self.changes.insert(hash.clone(), HashMap::new());
        }

        let changes = self.changes.get_mut(&hash).unwrap();
        for (k, v) in card_lock.changes.clone() {
            changes.insert(k, v);
        }

        card_lock.changes.clear();
    }
}
impl From<ResponsePacket> for Message {
    fn from(val: ResponsePacket) -> Self {
        Self::Text(serde_json::to_string(&val).unwrap().into())
    }
}
impl From<&ResponsePacket> for Message {
    fn from(val: &ResponsePacket) -> Self {
        Self::Text(serde_json::to_string(&val).unwrap().into())
    }
}
impl From<Packet> for ResponsePacket {
    fn from(value: Packet) -> Self {
        let mut response = Self::new(value.packet_type, Some(value.id), true, HashMap::default());
        response.request_data = value.data;
        response
    }
}

#[allow(
    clippy::significant_drop_tightening,
    reason = "We can't drop `peers` earlier than this."
)]
fn try_matchmake(peer_map: &PeerMap, queue: &Queue, games: &Games) {
    let Some((peer1, peer2)) = queue.try_matchmake() else {
        return;
    };

    // Two peers have been matchmade.
    log!(Level::Info, "Starting game for peers {peer1} and {peer2}");

    // TODO: Is unwrap safe here?
    let peers_lock = peer_map.get().unwrap();
    let (info1, sink1) = &peers_lock.get(&peer1).unwrap();
    let (info2, sink2) = &peers_lock.get(&peer2).unwrap();

    let game = games.add(peer_map, ((peer1, info1.clone()), (peer2, info2.clone())));
    log!(Level::Debug, "Games: {games:?}");

    // Set decks.
    if let Err(err) = Player::setup_deck_from_deckcode(&game, true) {
        // TODO: We should probably handle this somehow.
        log!(Level::Error, "{err}");
    }
    if let Err(err) = Player::setup_deck_from_deckcode(&game, false) {
        // TODO: We should probably handle this somehow.
        log!(Level::Error, "{err}");
    }

    let game_lock = game.lock().unwrap();
    log!(
        Level::Debug,
        "Deck 1: {:?}",
        game_lock.player1.lock().unwrap().deck
    );
    log!(
        Level::Debug,
        "Deck 2: {:?}",
        game_lock.player2.lock().unwrap().deck
    );
    drop(game_lock);

    log!(Level::Debug, "Sending opponent info");
    for i in 0..2 {
        let (peer, info, sink) = if i == 0 {
            // Send player 2 info to sink 1.
            (peer1, info2, sink1)
        } else {
            // Send player 1 info to sink 2.
            (peer2, info1, sink2)
        };

        // Send player info.
        let mut response =
            ResponsePacket::new("opponent_player_info".into(), None, false, HashMap::new());

        // TODO: Is unwrap safe here?
        let map = serde_json::to_value(info.lock().unwrap().clone()).unwrap();
        let map = map.as_object().unwrap().clone();
        for (k, v) in map {
            response.response_data.insert(k, v);
        }

        // FIXME: Unwrap here will probably poison the peer map.
        sink.unbounded_send(response.into()).unwrap();

        // Send `begin_game` packet.
        log!(Level::Debug, "Sending 'begin_game' packet");
        let game_lock = game.lock().unwrap();
        let response = ResponsePacket::new(
            "begin_game".into(),
            None,
            game_lock.is_peer_starting_player(peer),
            HashMap::new(),
        );
        // FIXME: This will probably poison the peer map.
        sink.unbounded_send(response.into()).unwrap();
    }

    drop(peers_lock);
    log!(Level::Debug, "Starting game");
    let mut response = ResponsePacket::new("begin_game".into(), None, false, HashMap::new());
    let game_lock = game.lock().unwrap();
    game_lock.start_game(&mut Some(&mut response));
}

fn handle_client_packet(
    peer_map: &PeerMap,
    queue: &Queue,
    games: &Games,
    peer: SocketAddr,
    packet: Packet,
) {
    log!(Level::Debug, "Handling packet {packet:?} from {peer}");

    // TODO: Is unwrap safe here?
    let peer_info = peer_map.get_entry(&peer).unwrap();

    let mut response: ResponsePacket = packet.into();

    if response.packet_type == "player_info" {
        let mut info_lock = peer_info.0.lock().unwrap();

        if let Some(username) = response.request_data.get("username") {
            info_lock.username = username.as_str().unwrap().into();
            log!(Level::Debug, "Renamed {peer} to: {}", info_lock.username);
        }
        if let Some(deckcode) = response.request_data.get("deckcode") {
            info_lock.deckcode = deckcode.as_str().unwrap().into();
            log!(
                Level::Debug,
                "Set {peer}'s deckcode to: {}",
                info_lock.deckcode
            );
        }

        // Respond.
        // NOTE: Include the automatically generated username if the packet didn't include one.
        response
            .response_data
            .insert("username".into(), info_lock.username.clone().into());
        peer_info.1.unbounded_send(response.into()).unwrap();
        return;
    } else if response.packet_type == "enter_queue" {
        // TODO: Block player from entering queue if they haven't sent their player info.
        queue.add(peer);

        // Respond.
        peer_info.1.unbounded_send(response.into()).unwrap();

        drop(peer_info);
        log!(Level::Debug, "Queue: {queue:?}");
        try_matchmake(peer_map, queue, games);
        return;
    }

    drop(peer_info);
    if games.peer_in_game(peer) {
        games.handle_client_packet(peer_map, peer, response);
    }
}

fn handle_disconnect(peer_map: &PeerMap, queue: &Queue, games: &Games, addr: SocketAddr) {
    let response = ResponsePacket::new(
        "end_game".into(),
        None,
        false,
        HashMap::from([("reason".into(), "opponent_disconnected".into())]),
    );
    games.send_message_by_peer(peer_map, addr, response.into());

    if let Some(opposing_peer) = games.get_opposing_peer(addr) {
        log!(
            Level::Info,
            "Game between {addr} and {opposing_peer} ended since {addr} disconnected."
        );

        // Automatically add opposing peer back to queue.
        // queue.add(opposing_peer);
        // try_matchmake(&peer_map, &queue, &games);
    }

    log!(Level::Debug, "{addr} disconnected");
    // TODO: Is unwrap safe here?
    peer_map.remove(&addr).unwrap();
    queue.remove(addr);
    games.remove_by_peer(addr);
    log!(Level::Debug, "Games: {games:?}");
}

async fn handle_connection(
    peer_map: PeerMap,
    queue: Queue,
    games: Games,
    raw_stream: TcpStream,
    addr: SocketAddr,
) {
    log!(Level::Debug, "Incoming TCP connection from: {addr}");

    let ws_stream = match tokio_tungstenite::accept_async(raw_stream).await {
        Ok(ws_stream) => ws_stream,
        Err(err) => {
            log!(Level::Error, "Could not accept connection. {err}");
            return;
        }
    };

    log!(Level::Debug, "WebSocket connection established: {addr}");
    let (tx, rx) = unbounded();

    // Insert the write part of this peer to the peer map.
    let player_info = PlayerInfo {
        username: format!("Anon {}", Ulid::generate()),
        ..Default::default()
    };
    // TODO: Is unwrap safe here?
    peer_map
        .insert(addr, (Arc::new(Mutex::new(player_info)), tx))
        .unwrap();

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        if msg.is_close() {
            return future::ok(());
        }

        // Peer sent a message.
        let message = match msg.to_text() {
            Ok(message) => message,
            Err(err) => {
                log!(Level::Error, "Peer {addr} sent invalid message.");
                return future::err(err);
            }
        };

        let packet: Packet = match serde_json::from_str(message) {
            Ok(value) => value,
            Err(err) => {
                log!(Level::Error, "Error when parsing message from {addr}");
                return future::err(tokio_tungstenite::tungstenite::Error::Utf8(format!(
                    "error when parsing message from {addr}. {err}"
                )));
            }
        };

        log!(Level::Debug, "Received a packet from {addr}: {packet:?}");
        handle_client_packet(&peer_map, &queue, &games, addr, packet);

        // let peers = peer_map.lock().unwrap();

        // Repeat message to all clients except for sender.
        // let broadcast_recipients = peers
        //     .iter()
        //     .filter(|(peer_addr, _)| peer_addr != &&addr)
        //     .map(|(_, ws_sink)| ws_sink);

        // for recp in broadcast_recipients {
        //     recp.unbounded_send(msg.clone()).unwrap();
        // }

        future::ok(())
    });

    let recieve_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, recieve_from_others);
    future::select(broadcast_incoming, recieve_from_others).await;

    // Peer disconnected
    handle_disconnect(&peer_map, &queue, &games, addr);
}

#[tokio::main]
pub async fn listen(port: u16) -> Result<(), Box<dyn Error>> {
    // TODO: Add reconnecting.
    // TODO: Add proper handling for CTRL+C.
    let addr = format!("127.0.0.1:{port}");
    // TODO: Make sure these global mutexes are never poisoned.
    let peer_map = PeerMap::default();
    let queue = Queue::default();
    let games = Games::default();

    let listener = TcpListener::bind(&addr).await?;
    log!(Level::Info, "Listening on {addr}");

    while let Ok((stream, addr)) = listener.accept().await {
        assert!(!peer_map.is_poisoned(), "Peer map is poisoned.");
        assert!(!queue.is_poisoned(), "Queue is poisoned.");
        assert!(!games.is_poisoned(), "Games is poisoned.");

        // TODO: Handle panics. Delete game / peer from queue. Automatically spawn new task to handle connection and send 'server_panicked' packet.
        tokio::spawn(handle_connection(
            peer_map.clone(),
            queue.clone(),
            games.clone(),
            stream,
            addr,
        ));
    }

    Ok(())
}
