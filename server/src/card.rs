use mlua::FromLua;
use mlua::prelude::*;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{lua::LuaVM, player::SharedPlayer};

pub type SharedCard = Arc<Mutex<Card>>;

#[derive(Debug, Default)]
pub struct Card {
    pub owner: Option<SharedPlayer>,
    pub name: String,
    pub cost: i32,
    pub id: String,
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
    }
}

impl FromLua for Card {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        let c = value.as_table().unwrap();

        let name = c.get::<String>("name").unwrap();
        let cost = c.get::<i32>("cost").unwrap();
        let id = c.get::<String>("id").unwrap();

        Ok(Self {
            name,
            cost,
            owner: None,
            // TODO: Set equal to name if no id.
            id,
        })
    }
}

impl Card {
    pub fn get_owner_name(&self) -> Option<String> {
        let owner_clone = self.owner.clone()?;
        let owner_lock = owner_clone.lock().unwrap();
        let info_lock = owner_lock.info.lock().unwrap();
        Some(info_lock.username.clone())
    }

    pub fn parse_from_file_content(lua_vm: &LuaVM, file_content: String) -> LuaResult<Self> {
        // TODO: Set proper name.
        lua_vm.lua.load(file_content).set_name("card").exec()?;
        let card = lua_vm.lua.globals().get::<Self>("card")?;
        Ok(card)
    }
}
