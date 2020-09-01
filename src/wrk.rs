use std::net::TcpListener;
use crate::state::State;
use std::sync::{RwLock, Arc};
use std::sync::mpsc::Sender;
use std::thread::park;

pub fn wrk(state: Arc<RwLock<State>>, sender: Sender<u32>) {
    let state = state.read().unwrap();
    if state.cluster_size == 0 {
        println!("Operating in single node cluster.");
        park();
        // don't spawn heartbeat thread
    } else {
        println!("Spawning heartbeat thread.");
    }
}

