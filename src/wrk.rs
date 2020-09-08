use crate::state::State;
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::thread::park;
use crate::seq_recovery::seq_recovery;
use crate::heartbeat::heartbeat;
use crate::internal::ThreadSignal;

pub fn wrk(state: Arc<RwLock<State>>, _sender: Sender<ThreadSignal>) {
    let mut state_ref = state.write().unwrap();
    if state_ref.get_cluster_size() > 0 {
        println!("Acquiring sequence number");
        let seq_num = seq_recovery(state_ref.get_neighbour_addrs(), state_ref.self_node_information.id);
        state_ref.sequence = seq_num;
        println!("Starting from sequence number: {}", seq_num);
    }

    drop(state_ref);
    let (monitor_sender, monitor_receiver): (Sender<ThreadSignal>, Receiver<ThreadSignal>) = mpsc::channel();

    // start heartbeat thread
    rayon::spawn(move || heartbeat(state.clone(), 5, 5, monitor_receiver));


    park();
}

