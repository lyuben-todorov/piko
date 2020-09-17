use crate::state::{State, Mode};
use std::sync::{RwLock, Arc};

use std::thread::park;
use crate::req::{push_state::push_state, seq_recovery::seq_recovery};


use log::{debug, error, info, trace, warn};








///
/// Tasked with maintaining protocol consistency
///
pub fn wrk(state: Arc<RwLock<State>>) {
    let mut state_ref = state.write().unwrap();

    let _local_id = state_ref.self_node_information.id;

    if state_ref.get_cluster_size() == 0 {}
    info!("Acquiring sequence number");
    let neighbours = state_ref.get_neighbour_addrs();
    let seq_num = seq_recovery(&neighbours);

    state_ref.sequence = seq_num;

    // Send state to neighbours
    push_state(&neighbours, Mode::Wrk);
    info!("Starting from sequence number: {}", seq_num);


    // release state lock
    drop(state_ref);


    park();
}

