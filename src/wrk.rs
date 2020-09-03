
use crate::state::State;
use std::sync::{RwLock, Arc};
use std::sync::mpsc::Sender;
use std::thread::park;

pub fn wrk(state: Arc<RwLock<State>>, _sender: Sender<u32>) {
    let state = state.read().unwrap();
    if state.get_size() == 0 {
        drop(state);
        println!("Parked on single node cluster.");
        park();
        // don't spawn heartbeat thread
    } else {
        drop(state);
        println!("Parked on multiple node cluster.");
        park();
    }
}

