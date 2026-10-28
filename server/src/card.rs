use mlua::FromLua;
use mlua::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::{fmt::Display, sync::Mutex};
use ulid::Ulid;

use crate::Shared;
use crate::game::SharedGame;
use crate::player::SharedPlayer;

#[derive(Debug, Default, Serialize, Clone)]
pub enum Location {
    #[default]
    Oblivion,
    Deck,
    Hand,
    Board,
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
    pub owner: Option<SharedPlayer>,
    pub is_starting_player: bool,
    pub name: String,
    pub cost: i32,
    pub text: String,
    pub id: String,
    pub location: Location,
    #[serde(skip)]
    pub onplay: Option<LuaFunction>,
    pub hash: Ulid,
    #[serde(skip)]
    pub changes: HashMap<String, Value>,
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
            location: self.location.clone(),
            onplay: self.onplay.clone(),
            hash: Ulid::generate(),
            changes: HashMap::new(),
        }
    }
}

impl LuaUserData for Card {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_set("name", |_, this, val| {
            this.set_name(val);
            Ok(())
        });

        fields.add_field_method_get("cost", |_, this| Ok(this.cost));
        fields.add_field_method_set("cost", |_, this, val| {
            this.set_cost(val);
            Ok(())
        });

        fields.add_field_method_get("text", |_, this| Ok(this.text.clone()));
        fields.add_field_method_set("text", |_, this, val| {
            this.set_text(val);
            Ok(())
        });

        fields.add_field_method_get("onplay", |_, this| Ok(this.onplay.clone()));
        fields.add_field_method_set("onplay", |_, this, val| {
            this.onplay = Some(val);
            Ok(())
        });
    }
}

impl FromLua for Card {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let c = value.as_table().unwrap();

        let name = c.get::<String>("name").unwrap();
        let cost = c.get::<i32>("cost").unwrap_or(0);
        let text = c.get::<String>("text").unwrap_or_else(|_| String::new());
        let id = c.get::<String>("id").unwrap_or_else(|_| name.clone());
        let onplay = c.get::<LuaFunction>("onplay").ok();

        Ok(Self {
            name,
            cost,
            text,
            id,
            onplay,
            hash: Ulid::generate(),
            ..Default::default()
        })
    }
}

impl Card {
    // TODO: Replace with enchantments?
    pub fn set_name(&mut self, name: String) {
        self.name = name.clone();
        self.changes.insert("name".into(), name.into());
    }
    pub fn set_cost(&mut self, cost: i32) {
        self.cost = cost;
        self.changes.insert("cost".into(), cost.into());
    }
    pub fn set_text(&mut self, text: String) {
        self.text = text.clone();
        self.changes.insert("text".into(), text.into());
    }
    pub fn set_location(&mut self, location: Location) {
        self.location = location.clone();
        self.changes
            .insert("location".into(), serde_json::to_value(location).unwrap());
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "Can't drop `game_lock` while `lua_vm` is in scope."
    )]
    pub fn parse_from_file_content(
        game: &SharedGame,
        file_content: String,
        file_name: String,
    ) -> LuaResult<Self> {
        let game_lock = game.lock().unwrap();
        let lua_vm = &game_lock.lua_vm;
        lua_vm.prepare_for_resource(&game_lock)?;

        let card = lua_vm.lua.load(file_content).set_name(file_name).eval()?;
        Ok(card)
    }

    pub fn get_owner_name(&self) -> Option<String> {
        let owner_clone = self.owner.clone()?;
        let owner_lock = owner_clone.lock().unwrap();
        let info_lock = owner_lock.info.lock().unwrap();
        Some(info_lock.username.clone())
    }

    pub fn remove_from_location(&self) {
        let Some(owner) = &self.owner else {
            return;
        };

        let mut owner_lock = owner.lock().unwrap();
        owner_lock.hand.erase(self);
        owner_lock.deck.erase(self);
        owner_lock.board.erase(self);
    }
}
