use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    card::SharedCard,
    netcode::net::Tx,
    player::{Player, SharedPlayer},
};

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

    pub fn send_to(&self, sink: &Tx) -> Result<(), futures_channel::mpsc::TrySendError<Message>> {
        sink.unbounded_send(self.into())
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
