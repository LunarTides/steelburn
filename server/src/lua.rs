use std::sync::MutexGuard;

use mlua::prelude::*;
use mlua::{Lua, Result};

use crate::game::Game;
use crate::player::LuaPlayer;

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

    pub fn prepare_for_card(&self, owner: LuaPlayer) -> Result<()> {
        let globals = self.lua.globals();
        globals.set("owner", owner.clone())?;

        Ok(())
    }

    pub fn prepare_for_resource(&self, game_lock: &MutexGuard<Game>) -> Result<()> {
        let globals = self.lua.globals();

        // TODO: Implement
        // let player1_lock = game_lock.player1.lock().unwrap();
        // globals.set("player1", player1_lock)?;
        // drop(player1_lock);

        Ok(())
    }
}
