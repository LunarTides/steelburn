use std::collections::{HashMap, hash_map::Entry};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    card::SharedCard,
    card_collection::CardCollection,
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
    #[serde(skip)]
    pub response_cards: HashMap<String, CardCollection>,
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
            response_cards: HashMap::new(),
            error: None,
            changes: HashMap::new(),
        }
    }

    pub fn add_card(&mut self, key: &str, card: SharedCard) {
        if let Entry::Vacant(c) = self.response_cards.entry(key.to_string()) {
            c.insert(CardCollection::from_vec(vec![card]));
            return;
        }

        let cards = self.response_cards.get_mut(key).unwrap();
        cards.push(card);
    }

    pub fn add_cards(&mut self, key: &str, cards: CardCollection) {
        self.response_cards.insert(key.to_string(), cards);
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
            self.use_card_changes_shared(player, &card);
        }
        for card in &player.hand {
            self.use_card_changes_shared(player, &card);
        }
        for card in &player.board {
            self.use_card_changes_shared(player, &card);
        }
    }

    pub fn use_player_changes_shared(&mut self, player: &SharedPlayer) {
        let mut player_lock = player.lock().unwrap();
        self.use_player_changes(&mut player_lock);
    }

    pub fn use_card_changes_shared(&mut self, player: &Player, card: &SharedCard) {
        // TODO: Don't send sensitive changes to the opposing peer.
        let mut card_lock = card.lock().unwrap();
        if card_lock.changes.is_empty() {
            return;
        }

        // FIXME: For some reason, opponent's cards include changes when they're played.
        if (!card_lock.has_sent_to_player1 && player.is_starting_player)
            || (!card_lock.has_sent_to_player2 && !player.is_starting_player)
        {
            // The player hasn't recieved any information about this card before. It is unknown, so don't send any changes.
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

    pub fn prepare_for_send(&self, player: &SharedPlayer) -> Self {
        let mut response = self.clone();
        let player_lock = player.lock().unwrap();
        let is_player_1 = player_lock.is_starting_player;

        for (key, cards) in &self.response_cards {
            let mut json_cards = Vec::new();

            for card in cards {
                let mut card_lock = card.lock().unwrap();

                if (card_lock.has_sent_to_player1 && is_player_1)
                    || (card_lock.has_sent_to_player2 && !is_player_1)
                {
                    // The card has already been sent before. Only send the hash.
                    json_cards.push(card_lock.hash.to_string().into());
                } else {
                    // The card hasn't been sent to this player before. Send the entire card.
                    let mut clone = card_lock.clone();
                    clone.hash = card_lock.hash;
                    json_cards.push(serde_json::to_value(clone).unwrap());
                }

                card_lock.has_sent_to_player1 = card_lock.has_sent_to_player1 || is_player_1;
                card_lock.has_sent_to_player2 = card_lock.has_sent_to_player2 || !is_player_1;
            }

            if json_cards.len() > 1 {
                // There's more than one card. Insert it as an array.
                response
                    .response_data
                    .insert(key.clone(), json_cards.into());
            } else if !json_cards.is_empty() {
                // There's only one card. Insert it as a single element, rather than an array.
                response
                    .response_data
                    .insert(key.clone(), json_cards.get(0).unwrap().clone());
            }
        }

        response.censor(&player_lock);
        response
    }

    pub fn censor(&mut self, player: &Player) {
        // TODO: Implement.
    }

    pub fn send_to(
        &self,
        sink: &Tx,
        player: Option<&SharedPlayer>,
    ) -> Result<(), futures_channel::mpsc::TrySendError<Message>> {
        let packet = if let Some(player) = player {
            &self.prepare_for_send(player)
        } else {
            self
        };

        sink.unbounded_send(packet.into())
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
