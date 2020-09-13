use crate::state::{State, Mode};
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::thread::park;
use crate::req::{push_state::push_state, seq_recovery::seq_recovery};
use crate::heartbeat::heartbeat;
use crate::internal::ThreadSignal;
use log::{debug, error, info, trace, warn};

pub fn wrk(state: Arc<RwLock<State>>, _sender: Sender<ThreadSignal>) {
    let mut state_ref = state.write().unwrap();

    if state_ref.get_cluster_size() > 0 {
        info!("Acquiring sequence number");
        let neighbours = state_ref.get_neighbour_addrs();
        let seq_num = seq_recovery(&neighbours);

        state_ref.sequence = seq_num;

        push_state(&neighbours, Mode::Wrk);
        info!("Starting from sequence number: {}", seq_num);
    }

    drop(state_ref);
    let (_monitor_sender, monitor_receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel();

    // start heartbeat thread
    rayon::spawn(move || heartbeat(state.clone(), 5, 5, monitor_receiver));


    park();
}

