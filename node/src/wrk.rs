use crate::state::{State, Mode};
use std::sync::{RwLock, Arc, Mutex};


use crate::req::{push_state::push_state, seq_recovery::seq_recovery};

use log::{debug, error, info, trace, warn};
use std::sync::mpsc::Receiver;
use crate::proto::{Pledge, ResourceRequest};
use std::collections::{BinaryHeap, HashMap};
use crate::client::Client;

///
/// Tasked with maintaining protocol consistency
///
pub fn wrk(state: Arc<RwLock<State>>, pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
           recv: &Receiver<Pledge>, _client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>>) {
    let mut state_ref = state.write().unwrap();

    let self_id = state_ref.id;

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

    for pledge in recv.iter() {
        match pledge {
            Pledge::Kuchek => {
                let q_ref = pledge_queue.lock().unwrap();
                let pledge = q_ref.peek().unwrap();

                if pledge.owner == self_id {
                    println!("Consuming resource! node {} message {}", pledge.owner, String::from_utf8_lossy(&pledge.message_hash));
                    // consume resource
                }
            }
            Pledge::ResourceRelease(_rel) => {
                // propagate release to clients
            }
        }
    }
}

