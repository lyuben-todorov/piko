use std::net::TcpListener;
use crate::state::State;
use std::sync::{RwLock, Arc};

pub fn wrk(state: Arc<RwLock<State>>) {
    let state = state.read().unwrap();
    if state.cluster_size == 0 {
        println!("Operating in single node cluster.");
        // don't spawn heartbeat thread
    } else {
        println!("Spawning heartbeat thread.");
    }
}

