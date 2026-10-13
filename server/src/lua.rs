use std::ops::Deref;
use std::sync::MutexGuard;

use mlua::prelude::*;
use mlua::{Lua, Result};

use crate::card::Card;
use crate::game::Game;

#[derive(Debug)]
pub struct LuaVM {
    pub lua: Lua,
}

impl LuaVM {
    pub fn start() -> Result<Self> {
        let lua = Lua::new();
        lua.sandbox(true)?;

        Ok(Self { lua })
    }

    pub fn prepare_for_resource(&self, game_lock: &MutexGuard<Game>) -> Result<()> {
        let globals = self.lua.globals();

        let player1_lock = game_lock.player1.lock().unwrap();
        globals.set("player1", player1_lock.deref().clone())?;
        drop(player1_lock);

        globals.set("card", Card::default())?;

        Ok(())
    }
}
