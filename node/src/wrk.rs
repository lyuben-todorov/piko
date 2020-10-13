use crate::state::{State, Mode};
use std::sync::{RwLock, Arc, Mutex};


use crate::req::{push_state::push_state, seq_recovery::seq_recovery};

use log::{info};
use std::sync::mpsc::Receiver;
use crate::proto::{Pledge, ResourceRequest, ResourceRelease};
use std::collections::{BinaryHeap, HashMap};
use crate::client::Client;
use crate::req::publish::pub_rel;

// Tasked with maintaining protocol consistency
pub fn wrk(state: Arc<RwLock<State>>, pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
           recv: &Receiver<Pledge>, _client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>>,
           pending_messages: Arc<Mutex<HashMap<u16, ResourceRelease>>>) {
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
            Pledge::Check => {
                info!("Checking resource queue");
                let mut q_ref = pledge_queue.lock().unwrap();
                let pledge = q_ref.peek().unwrap();

                if pledge.owner == self_id {
                    let pledge = q_ref.pop().unwrap();

                    info!("Consuming resource! node {} message {}", pledge.owner, pledge.shorthand);

                    let mut messages = pending_messages.lock().unwrap();
                    let message = messages.remove(&pledge.shorthand).unwrap();

                    let state = state.read().unwrap();
                    pub_rel(&state.get_neighbour_addrs(), message);
                }
            }
            Pledge::ResourceRelease(rel) => {
                let mut q_ref = pledge_queue.lock().unwrap();
                let pledge = q_ref.peek().unwrap();
                if pledge.owner == rel.owner {
                    let pledge = q_ref.pop().unwrap();
                    info!("Neighbour exited CS! node {} message {}", pledge.owner, String::from_utf8(rel.message.message).unwrap());

                }
            }
        }
    }
}

