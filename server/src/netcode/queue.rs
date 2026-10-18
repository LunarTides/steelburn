use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Default, Clone, Debug)]
pub struct Queue {
    data: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Queue {
    pub fn add(&self, addr: SocketAddr) {
        self.data.lock().unwrap().push(addr);
    }

    pub fn remove(&self, addr: SocketAddr) -> Option<usize> {
        let mut data = self.data.lock().unwrap();
        let queue_index = data.iter().position(|x| *x == addr)?;
        data.remove(queue_index);
        drop(data);
        Some(queue_index)
    }

    pub fn is_poisoned(&self) -> bool {
        self.data.is_poisoned()
    }

    pub fn try_matchmake(&self) -> Option<(SocketAddr, SocketAddr)> {
        let mut data = self.data.lock().unwrap();
        if data.len() < 2 {
            return None;
        }

        // TODO: More advanced matchmaking code.
        Some((data.pop().unwrap(), data.pop().unwrap()))
    }
}
