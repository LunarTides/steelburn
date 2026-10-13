use mlua::FromLua;
use mlua::prelude::*;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use ulid::Ulid;

use crate::game::SharedGame;
use crate::player::SharedPlayer;

pub type SharedCard = Arc<Mutex<Card>>;

impl From<Card> for SharedCard {
    fn from(value: Card) -> Self {
        Self::new(Mutex::new(value))
    }
}

#[derive(Debug, Default)]
pub struct Card {
    pub owner: Option<SharedPlayer>,
    pub name: String,
    pub cost: i32,
    pub id: String,
    pub onplay: Option<LuaFunction>,
    pub hash: Ulid,
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
            name: self.name.clone(),
            cost: self.cost,
            id: self.id.clone(),
            owner: self.owner.clone(),
            onplay: self.onplay.clone(),
            hash: Ulid::generate(),
        }
    }
}

impl LuaUserData for Card {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_set("name", |_, this, val| {
            this.name = val;
            Ok(())
        });

        fields.add_field_method_get("cost", |_, this| Ok(this.cost));
        fields.add_field_method_set("cost", |_, this, val| {
            this.cost = val;
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
        let cost = c.get::<i32>("cost").unwrap();
        let id = c.get::<String>("id").unwrap_or_else(|_| name.clone());
        let onplay = c.get::<LuaFunction>("onplay").ok();

        Ok(Self {
            name,
            cost,
            // TODO: Set equal to name if no id.
            id,
            onplay,
            hash: Ulid::generate(),
            ..Default::default()
        })
    }
}

impl Card {
    #[allow(
        clippy::significant_drop_tightening,
        reason = "Can't drop `game_lock` while `lua_vm` is in scope."
    )]
    pub fn parse_from_file_content(game: &SharedGame, file_content: String) -> LuaResult<Self> {
        let game_lock = game.lock().unwrap();
        let lua_vm = &game_lock.lua_vm;
        lua_vm.prepare_for_resource(&game_lock)?;

        // TODO: Set proper name.
        let card = lua_vm.lua.load(file_content).set_name("card").eval()?;
        Ok(card)
    }

    pub fn get_owner_name(&self) -> Option<String> {
        let owner_clone = self.owner.clone()?;
        let owner_lock = owner_clone.lock().unwrap();
        let info_lock = owner_lock.info.lock().unwrap();
        Some(info_lock.username.clone())
    }
}
