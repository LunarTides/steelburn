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

use crate::netcode::{games::Games, queue::Queue};

type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, (SharedPlayerInfo, Tx)>>>;

#[derive(Default, Debug)]
pub struct PlayerInfo {
    pub username: String,
}

pub type SharedPlayerInfo = Arc<Mutex<PlayerInfo>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    #[serde(rename = "type")]
    pub packet_type: String,
    pub data: Map<String, Value>,
}

fn try_matchmake(peer_map: &PeerMap, queue: &Queue, games: &Games) {
    if let Some((peer1, peer2)) = queue.try_matchmake() {
        // Two peers have been matchmade.
        log!(Level::Info, "Starting game for peers {peer1} and {peer2}");

        let peers = peer_map.lock().unwrap();
        let info_1 = &peers.get(&peer1).unwrap().0;
        let info_2 = &peers.get(&peer2).unwrap().0;

        games.add(((peer1, info_1.clone()), (peer2, info_2.clone())));
        drop(peers);

        games.send_message_by_peer(
            peer_map,
            peer1,
            &Message::Text("You have been added into a game.".into()),
        );
    }
}

fn handle_client_packet(peer_map: &PeerMap, games: &Games, peer: SocketAddr, packet: &Packet) {
    log!(Level::Debug, "Handling packet {packet:?} from {peer}");

    if packet.packet_type == "player_info" {
        let mut peers = peer_map.lock().unwrap();
        let mut info_lock = peers.get_mut(&peer).unwrap().0.lock().unwrap();

        if let Some(username) = packet.data.get("username") {
            info_lock.username = username.to_string();
            drop(info_lock); // remove when adding more conditions later on
            log!(Level::Debug, "Renamed {peer} to {username}");
        }
        return;
    }

    if games.peer_in_game(peer) {
        games.handle_client_packet(peer_map, peer, packet);
    }
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

    queue.add(addr);
    try_matchmake(&peer_map, &queue, &games);

    log!(Level::Debug, "Queue: {:?}", queue.data);
    log!(Level::Debug, "Games: {:?}", games.data);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
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
        handle_client_packet(&peer_map, &games, addr, &packet);

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
    games.send_message_by_peer(&peer_map, addr, &Message::Text("Peer disconnected".into()));
    if let Some(opposing_peer) = games.get_opposing_peer(addr) {
        log!(
            Level::Info,
            "Game between {addr} and {opposing_peer} ended since {addr} disconnected."
        );

        // Add opposing peer back to queue.
        queue.add(opposing_peer);
        try_matchmake(&peer_map, &queue, &games);
    }

    log!(Level::Debug, "{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
    queue.remove(addr);
    games.remove_by_peer(addr);
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
