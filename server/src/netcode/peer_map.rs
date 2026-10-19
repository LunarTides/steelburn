use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::netcode::net::{Poisoned, SharedPlayerInfo, Tx};

type Data = HashMap<SocketAddr, (SharedPlayerInfo, Tx)>;
type SharedData = Arc<Mutex<Data>>;

#[derive(Default, Clone, Debug)]
pub struct PeerMap {
    data: SharedData,
}

impl PeerMap {
    pub fn get(&self) -> Result<MutexGuard<'_, Data>, Poisoned<'_, Data>> {
        self.data.lock()
    }

    pub fn get_entry(
        &self,
        peer: &SocketAddr,
    ) -> Result<(SharedPlayerInfo, Tx), Box<dyn Error + '_>> {
        let mut entries = self.get()?;
        let entry = entries.get_mut(peer);
        entry.map_or_else(
            || Err(Box::from(format!("No info exists for peer {peer}"))),
            |entry| Ok(entry.clone()),
        )
    }

    pub fn is_poisoned(&self) -> bool {
        self.data.is_poisoned()
    }

    pub fn insert(
        &self,
        peer: SocketAddr,
        info: (SharedPlayerInfo, Tx),
    ) -> Result<(), Poisoned<'_, Data>> {
        self.get()?.insert(peer, info);
        Ok(())
    }

    pub fn remove(&self, peer: &SocketAddr) -> Result<(), Poisoned<'_, Data>> {
        self.data.lock()?.remove(peer);
        Ok(())
    }
}
