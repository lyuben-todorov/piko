use crate::state::{State, Mode};
use std::sync::{RwLock, Arc, Mutex};


use crate::req::{push_state::push_state, seq_recovery::seq_recovery};

use log::{info, debug, error};
use std::sync::mpsc::{Receiver};
use crate::proto::{ResourceRequest, ResourceRelease};
use std::collections::{BinaryHeap, HashMap};

use crate::req::publish::pub_rel;
use std::time::Duration;


static SLEEP_TIME_MILLIS: u64 = 10;

// Tasked with maintaining protocol consistency
pub fn wrk(state: Arc<RwLock<State>>, resource_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
           recv: &Receiver<ResourceRelease>, pending_messages: Arc<Mutex<HashMap<u16, (ResourceRelease, bool)>>>) {
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


    loop {
        let q_ref = &resource_queue.clone();
        let q_lock = q_ref.lock().unwrap();
        let (req_owner, req_key) = match q_lock.peek() {
            None => {
                // Queue was empty
                drop(q_lock);
                std::thread::sleep(Duration::from_millis(SLEEP_TIME_MILLIS));
                continue;
            }
            Some(req) => {
                (req.owner, req.shorthand)
            }
        };
        // debug!("Current req: {} Me: {}", req.owner, self_id);
        drop(q_lock);
        if req_owner == self_id && is_acknowledged(pending_messages.clone(), req_key) {
            // begin executing CS
            let mut q_lock = q_ref.lock().unwrap();
            let resource = q_lock.pop().unwrap();
            info!("Entering CS! node {} message {}", resource.owner, resource.shorthand);
            let mut messages = pending_messages.lock().unwrap();
            let message = messages.remove(&resource.shorthand).unwrap();

            // drop before slow ops
            drop(q_lock);
            let state = state.read().unwrap();

            pub_rel(&state.get_neighbour_addrs(), message.0);
        } else {
            // gather resource releases
            let mut releases: usize = 0;
            let q_lock = q_ref.lock().unwrap();
            let req = q_lock.peek().unwrap();
            let owner = req.owner;
            drop(q_lock);
            for rel in recv.try_iter() {
                releases += 1;
                debug!("{} {}", owner, rel.owner);
                if owner == rel.owner {
                    let mut q_lock = q_ref.lock().unwrap();
                    let pledge = q_lock.pop().unwrap();
                    info!("Neighbour exited CS! node {} message {}", pledge.owner, String::from_utf8(rel.message.message).unwrap());
                    drop(q_lock);
                } else {
                    error!("Neighbour tried entering CS without lock!");
                }
            }
            if releases == 0 {
                std::thread::sleep(Duration::from_millis(SLEEP_TIME_MILLIS));
            }
        }
    }
}

fn is_acknowledged(map: Arc<Mutex<HashMap<u16, (ResourceRelease, bool)>>>, rel_key: u16) -> bool {
    const TRYOUTS: u8 = 3;
    let mut response = false;

    // try a few times
    for _i in 0..TRYOUTS {
        let map = map.lock().unwrap();
        response = match map.get(&rel_key) {
            Some(rel) => { rel.1 }
            _ => {
                false
            }
        };
        drop(map);
        if !response { std::thread::sleep(Duration::from_millis(10)); }
    }
    response
}