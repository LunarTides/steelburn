use std::collections::HashMap;

use mlua::prelude::*;
use mlua::{Lua, Result};
use serde_json::{Map, Value};

use crate::card::{Card, SharedCard};
use crate::netcode::packet::ResponsePacket;
use crate::player::SharedPlayer;

#[derive(Debug, Default, Clone)]
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
        // FIXME: For some ungodly reason, this doesn't work.
        // globals.set("card", card)?;
        globals.set("self", card)?;
        globals.set("owner", owner)?;

        Ok(())
    }

    pub fn prepare_for_resource(&self) -> Result<()> {
        let globals = self.lua.globals();

        let card = LuaCard::from(SharedCard::from(Card::default()));
        globals.set("card", card)?;

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

        fields.add_field_method_get("opponent", |_, this| {
            Ok(<SharedPlayer as Into<LuaPlayer>>::into(
                this.player.lock().unwrap().get_opponent(),
            ))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("send_packet", |_, this, value: (String, LuaTable)| {
            let is_starting_player = this.player.lock().unwrap().is_starting_player;

            let mut data = HashMap::new();
            for pair in value.1.pairs() {
                let (key, value): (String, LuaValue) = pair?;

                data.insert(key, serde_json::to_value(value).unwrap());
            }

            let mut packet = ResponsePacket::new(value.0, None, is_starting_player, data);
            packet.send_to(&mut this.player.lock().unwrap()).unwrap();
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

        fields.add_field_method_get("type", |_, this| {
            Ok(this.card.lock().unwrap().card_type.clone())
        });
        fields.add_field_method_set("type", |_, this, val| {
            this.card.lock().unwrap().set_type(val);
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
