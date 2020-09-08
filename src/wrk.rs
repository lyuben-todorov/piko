use crate::state::State;
use std::sync::{RwLock, Arc};
use std::sync::mpsc::Sender;
use std::thread::park;
use crate::seq_recovery::seq_recovery;

pub fn wrk(state: Arc<RwLock<State>>, _sender: Sender<u32>) {
    let state_ref = state.write().unwrap();
    if state_ref.get_cluster_size() > 0 {
        println!("Aquiring sequence number");
        let seq_num = seq_recovery(state);
        state_ref.sequence = seq_num;
        println!("Starting from sequence number: {}", seq_num);
        park();
    }

    drop(state_ref);
    park();
}

