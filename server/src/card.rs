use mlua::FromLua;
use mlua::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::{fmt::Display, sync::Mutex};
use ulid::Ulid;

use crate::Shared;
use crate::lua::LuaVM;
use crate::player::Player;
use crate::player::SharedPlayer;

#[derive(Debug, Default, Serialize, Clone, PartialEq, Eq)]
pub enum Location {
    #[default]
    Oblivion,
    Deck,
    Hand,
    Board,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Type {
    #[default]
    Minion,
    Spell,
}
impl FromLua for Type {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let card_type = value.to_string()?;

        // serde_json::from_str::<Self>(&card_type).or_else(|x| Err(LuaError::runtime(x.to_string())))

        match card_type.to_lowercase().as_str() {
            "minion" => Ok(Self::Minion),
            "spell" => Ok(Self::Spell),
            unknown => Err(LuaError::runtime(format!("No such card type: '{unknown}'"))),
        }
    }
}
impl IntoLua for Type {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        match self {
            Self::Minion => Ok(LuaValue::String(lua.create_string("minion").unwrap())),
            Self::Spell => Ok(LuaValue::String(lua.create_string("spell").unwrap())),
        }
    }
}

pub type SharedCard = Shared<Card>;

impl From<Card> for SharedCard {
    fn from(value: Card) -> Self {
        Self::new(Mutex::new(value))
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Card {
    #[serde(skip)]
    /// The player who owns this card.
    pub owner: Option<SharedPlayer>,
    /// Whether or not this card is owned by the starting player.
    pub is_starting_player: bool,
    pub name: String,
    pub cost: i32,
    pub text: String,
    pub id: String,
    #[serde(rename = "type")]
    pub card_type: Type,
    pub location: Location,
    #[serde(skip)]
    pub onplay: Option<LuaFunction>,
    pub hash: Ulid,
    #[serde(skip)]
    /// Whether or not this card has been sent to the starting player's client.
    pub has_sent_to_player1: bool,
    #[serde(skip)]
    /// Whether or not this card has been sent to the coin player's client.
    pub has_sent_to_player2: bool,
    #[serde(skip)]
    /// Field that have changed since the last packet.
    pub changes: HashMap<String, Value>,
    #[serde(skip)]
    pub lua_vm: LuaVM,

    pub attack: i64,
    pub health: i64,
    pub exhausted: bool,
}

impl Display for Card {
    #[allow(clippy::significant_drop_tightening)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.get_owner_name();
        write!(
            f,
            "{} belonging to {}",
            self.name,
            name.unwrap_or_else(|| "Nobody".into())
        )
    }
}

impl Clone for Card {
    fn clone(&self) -> Self {
        Self {
            is_starting_player: self.is_starting_player,
            name: self.name.clone(),
            cost: self.cost,
            text: self.text.clone(),
            id: self.id.clone(),
            owner: self.owner.clone(),
            card_type: self.card_type.clone(),
            location: self.location.clone(),
            onplay: self.onplay.clone(),
            hash: Ulid::generate(),
            has_sent_to_player1: false,
            has_sent_to_player2: false,
            changes: HashMap::new(),
            lua_vm: self.lua_vm.clone(),
            attack: self.attack,
            health: self.health,
            exhausted: true,
        }
    }
}

impl FromLua for Card {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let c = value.as_userdata().unwrap();

        let name = c.get::<String>("name").unwrap();
        let card_type = c.get::<Type>("type").unwrap();
        let cost = c.get::<i32>("cost").unwrap_or_default();
        let text = c.get::<String>("text").unwrap_or_else(|_| String::new());
        let id = c.get::<String>("id").unwrap_or_else(|_| name.clone());
        let onplay = c.get::<LuaFunction>("onplay").ok();

        let attack = c.get::<i64>("attack").unwrap_or_default();
        let health = c.get::<i64>("health").unwrap_or_default();

        Ok(Self {
            name,
            card_type,
            cost,
            text,
            id,
            onplay,
            attack,
            health,
            hash: Ulid::generate(),
            ..Default::default()
        })
    }
}

impl Card {
    /// Creates a new card from a lua file.
    pub fn parse_from_file_content(file_content: String, file_name: String) -> LuaResult<Self> {
        let lua_vm = LuaVM::start()?;
        lua_vm.prepare_for_resource()?;

        lua_vm.lua.load(file_content).set_name(file_name).exec()?;
        let mut card: Self = lua_vm.lua.globals().get("card")?;
        card.lua_vm = lua_vm;

        Ok(card)
    }

    // TODO: Replace with enchantments?
    pub fn set_name(&mut self, name: String) {
        if self.name == name {
            return;
        }
        self.name.clone_from(&name);
        self.changes.insert("name".into(), name.into());
    }
    pub fn set_cost(&mut self, cost: i32) {
        if self.cost == cost {
            return;
        }
        self.cost = cost;
        self.changes.insert("cost".into(), cost.into());
    }
    pub fn set_text(&mut self, text: String) {
        if self.text == text {
            return;
        }
        self.text.clone_from(&text);
        self.changes.insert("text".into(), text.into());
    }
    pub fn set_type(&mut self, card_type: Type) {
        if self.card_type == card_type {
            return;
        }
        self.card_type.clone_from(&card_type);
        self.changes
            .insert("type".into(), serde_json::to_value(card_type).unwrap());
    }
    pub fn set_location(&mut self, location: Location) {
        if self.location == location {
            return;
        }
        self.location = location.clone();
        self.changes
            .insert("location".into(), serde_json::to_value(location).unwrap());
    }
    pub fn set_attack(&mut self, attack: i64) {
        if self.attack == attack {
            return;
        }
        self.attack = attack;
        self.changes.insert("attack".into(), attack.into());
    }
    pub fn set_health(&mut self, health: i64) {
        if self.health == health {
            return;
        }
        self.health = health;
        self.changes.insert("health".into(), health.into());

        if self.health <= 0 {
            self.destroy();
        }
    }
    pub fn set_exhausted(&mut self, exhausted: bool) {
        if self.exhausted == exhausted {
            return;
        }
        self.exhausted = exhausted;
        self.changes.insert("exhausted".into(), exhausted.into());
    }

    /// Returns the name of the owner, if the card has an owner. This locks the owner mutex.
    pub fn get_owner_name(&self) -> Option<String> {
        let owner_clone = self.owner.clone()?;
        let owner_lock = owner_clone.lock().unwrap();
        let info_lock = owner_lock.info.lock().unwrap();
        Some(info_lock.username.clone())
    }

    /// Removes the card from wherever it is.
    ///
    /// This tries removing the card from the owner's hand, deck, and board. This locks the owner mutex.
    pub fn remove_from_location(&self) {
        let Some(owner) = &self.owner else {
            return;
        };

        let mut owner_lock = owner.lock().unwrap();
        owner_lock.hand.erase(self);
        owner_lock.deck.erase(self);
        owner_lock.board.erase(self);
    }

    /// Assigns this card a new random hash.
    pub fn randomize_hash(&mut self) {
        self.hash = Ulid::generate();
    }

    pub fn destroy(&mut self) {
        self.set_location(Location::Oblivion);
    }

    pub fn attack_card(&mut self, target: &mut Self) -> bool {
        if self.exhausted {
            return false;
        }

        // Trade blows.
        // NOTE: Wrap for funsies :)
        self.set_health(self.health.wrapping_sub(target.attack));
        target.set_health(target.health.wrapping_sub(self.attack));

        self.set_exhausted(true);
        true
    }
    pub fn attack_player(&mut self, target: &mut Player) {}
}
