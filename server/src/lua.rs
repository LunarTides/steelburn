use std::sync::MutexGuard;

use mlua::prelude::*;
use mlua::{Lua, Result};

use crate::card::SharedCard;
use crate::game::Game;
use crate::player::SharedPlayer;

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

    pub fn prepare_for_card(&self, card: LuaCard, owner: LuaPlayer) -> Result<()> {
        let globals = self.lua.globals();
        globals.set("self", card)?;
        globals.set("owner", owner)?;

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

#[derive(Clone)]
pub struct LuaPlayer {
    pub player: SharedPlayer,
}

// TODO: Use mlua macros instead.
impl LuaUserData for LuaPlayer {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("is_starting_player", |_, this| {
            Ok(this.player.lock().unwrap().is_starting_player)
        });

        fields.add_field_method_get("username", |_, this| {
            Ok(this
                .player
                .lock()
                .unwrap()
                .info
                .lock()
                .unwrap()
                .username
                .clone())
        });
        fields.add_field_method_set("username", |_, this, val| {
            this.player.lock().unwrap().set_username(val);
            Ok(())
        });

        fields.add_field_method_get("mana", |_, this| Ok(this.player.lock().unwrap().mana));
        fields.add_field_method_set("mana", |_, this, val| {
            this.player.lock().unwrap().set_mana(val);
            Ok(())
        });
    }
}

// impl FromLua for Player {
//     fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
//         match value {
//             LuaValue::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
//             _ => unreachable!(),
//         }
//     }
// }

impl From<SharedPlayer> for LuaPlayer {
    fn from(value: SharedPlayer) -> Self {
        Self { player: value }
    }
}

pub struct LuaCard {
    pub card: SharedCard,
}

impl LuaUserData for LuaCard {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, this| Ok(this.card.lock().unwrap().name.clone()));
        fields.add_field_method_set("name", |_, this, val| {
            this.card.lock().unwrap().set_name(val);
            Ok(())
        });

        fields.add_field_method_get("cost", |_, this| Ok(this.card.lock().unwrap().cost));
        fields.add_field_method_set("cost", |_, this, val| {
            this.card.lock().unwrap().set_cost(val);
            Ok(())
        });

        fields.add_field_method_get("text", |_, this| Ok(this.card.lock().unwrap().text.clone()));
        fields.add_field_method_set("text", |_, this, val| {
            this.card.lock().unwrap().set_text(val);
            Ok(())
        });

        fields.add_field_method_get("onplay", |_, this| {
            Ok(this.card.lock().unwrap().onplay.clone())
        });
        fields.add_field_method_set("onplay", |_, this, val| {
            this.card.lock().unwrap().onplay = Some(val);
            Ok(())
        });
    }
}

impl From<SharedCard> for LuaCard {
    fn from(value: SharedCard) -> Self {
        Self { card: value }
    }
}
