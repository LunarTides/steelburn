use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::{StreamExt, TryStreamExt, future, pin_mut};
use log::{Level, log};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    netcode::{games::Games, queue::Queue},
    player::Player,
};

type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, (SharedPlayerInfo, Tx)>>>;
pub type SharedPlayerInfo = Arc<Mutex<PlayerInfo>>;

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
    pub data: Map<String, Value>,
}
impl Packet {
    pub const fn new(packet_type: String, data: Map<String, Value>) -> Self {
        Self { packet_type, data }
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

pub fn create_packet_message(json: &str) -> Message {
    serde_json::from_str::<Packet>(json).unwrap().into()
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

    let peers_lock = peer_map.lock().unwrap();
    let (info1, sink1) = &peers_lock.get(&peer1).unwrap();
    let (info2, sink2) = &peers_lock.get(&peer2).unwrap();

    let game = games.add(peer_map, ((peer1, info1.clone()), (peer2, info2.clone())));
    log!(Level::Debug, "Games: {:?}", games.data);

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
        let json = r#"
                {
                    "type": "opponent_player_info",
                    "data": {}
                }
            "#;

        let mut response_packet: Packet = serde_json::from_str(json).unwrap();
        // Put the PlayerInfo into the packet.
        response_packet.data.clone_from(
            serde_json::to_value(info.lock().unwrap().clone())
                .unwrap()
                .as_object()
                .unwrap(),
        );
        sink.unbounded_send(response_packet.into()).unwrap();

        // Send `begin_game` packet.
        log!(Level::Debug, "Sending 'begin_game' packet");
        let json = r#"
            {
                "type": "begin_game",
                "data": {
                    "starting_player": false
                }
            }
        "#;

        let mut response_packet: Packet = serde_json::from_str(json).unwrap();
        let game_lock = game.lock().unwrap();
        response_packet.data.insert(
            "starting_player".into(),
            serde_json::to_value(
                game_lock
                    .get_player_from_peer(peer)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .is_starting_player,
            )
            .unwrap(),
        );
        sink.unbounded_send(response_packet.into()).unwrap();
    }

    drop(peers_lock);
    log!(Level::Debug, "Starting game");
    let game_lock = game.lock().unwrap();
    game_lock.start_game();
}

fn handle_client_packet(
    peer_map: &PeerMap,
    queue: &Queue,
    games: &Games,
    peer: SocketAddr,
    packet: &Packet,
) {
    log!(Level::Debug, "Handling packet {packet:?} from {peer}");

    if packet.packet_type == "player_info" {
        let mut peers = peer_map.lock().unwrap();
        let mut info_lock = peers.get_mut(&peer).unwrap().0.lock().unwrap();

        if let Some(username) = packet.data.get("username") {
            info_lock.username = username.as_str().unwrap().into();
            log!(Level::Debug, "Renamed {peer} to: {}", info_lock.username);
        }
        if let Some(deckcode) = packet.data.get("deckcode") {
            info_lock.deckcode = deckcode.as_str().unwrap().into();
            log!(
                Level::Debug,
                "Set {peer}'s deckcode to: {}",
                info_lock.deckcode
            );
        }

        drop(info_lock); // remove when adding more conditions later on
        drop(peers);
        return;
    } else if packet.packet_type == "enter_queue" {
        queue.add(peer);
        log!(Level::Debug, "Queue: {:?}", queue.data);
        try_matchmake(peer_map, queue, games);
        return;
    }

    if games.peer_in_game(peer) {
        games.handle_client_packet(peer_map, peer, packet);
    }
}

fn handle_disconnect(peer_map: &PeerMap, queue: &Queue, games: &Games, addr: SocketAddr) {
    let json = r#"
        {
            "type": "end_game",
            "data": {
                "reason": "peer_disconnected"
            }
        }
    "#;
    let message = create_packet_message(json);
    games.send_message_by_peer(peer_map, addr, message);

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
    peer_map.lock().unwrap().remove(&addr);
    queue.remove(addr);
    games.remove_by_peer(addr);
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

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map
        .lock()
        .unwrap()
        .insert(addr, (Arc::new(Mutex::new(PlayerInfo::default())), tx));

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
        handle_client_packet(&peer_map, &queue, &games, addr, &packet);

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
    let state = PeerMap::new(Mutex::new(HashMap::new()));
    let queue = Queue::default();
    let games = Games::default();

    let listener = TcpListener::bind(&addr).await?;
    log!(Level::Info, "Listening on {addr}");

    while let Ok((stream, addr)) = listener.accept().await {
        // TODO: Handle panics. Delete game / peer from queue. Automatically spawn new task to handle connection and send 'server_panicked' packet.
        tokio::spawn(handle_connection(
            state.clone(),
            queue.clone(),
            games.clone(),
            stream,
            addr,
        ));
    }

    Ok(())
}
