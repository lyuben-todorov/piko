use crate::state::{State, Mode};
use std::sync::{RwLock, Arc, Mutex};


use crate::req::{push_state::push_state, seq_recovery::seq_recovery};

use log::{info, debug};
use std::sync::mpsc::{Receiver, TryRecvError};
use crate::proto::{Pledge, ResourceRequest, ResourceRelease};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use crate::client::Client;
use crate::req::publish::pub_rel;
use std::time::Duration;
use rand::thread_rng;

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
    // drop(state_ref);


    loop {
        let q_ref = &resource_queue.clone();
        debug!("Main lock");
        let mut q_ref = q_ref.lock().unwrap();
        let req = match q_ref.peek() {
            None => {
                // Queue was empty
                drop(q_ref);
                // println!("asd");
                std::thread::sleep(Duration::from_millis(500));
                debug!("Main rel");
                continue;
            }
            Some(req) => {
                req
            }
        };
        debug!("Current req: {} Me: {}", req.owner, self_id);
        if req.owner == self_id && is_acknowledged(pending_messages.clone(), req) {
            // begin executing CS
            let resource = q_ref.pop().unwrap();
            info!("Entering CS! node {} message {}", resource.owner, resource.shorthand);
            let mut messages = pending_messages.lock().unwrap();
            let message = messages.remove(&resource.shorthand).unwrap();
            let state = state.read().unwrap();
            pub_rel(&state.get_neighbour_addrs(), message.0);
        } else {
            // gather resource releases
            println!("Gathering resource releases");
            let mut releases: usize = 0;
            for rel in recv.try_iter() {
                releases += 1;
                let req = q_ref.peek().unwrap();
                if req.owner == rel.owner {
                    let pledge = q_ref.pop().unwrap();
                    info!("Neighbour exited CS! node {} message {}", pledge.owner, String::from_utf8(rel.message.message).unwrap());
                }
            }
            if releases == 0 {
                std::thread::sleep(Duration::from_millis(450));
            }
        }

        drop(q_ref);
        debug!("Main rel");
        std::thread::sleep(Duration::from_millis(500));

    }
}

fn is_acknowledged(map: Arc<Mutex<HashMap<u16, (ResourceRelease, bool)>>>, req: &ResourceRequest) -> bool {
    const TRYOUTS: u8 = 3;
    let mut response = false;

    // try a few times
    for i in 0..TRYOUTS {
        let map = map.lock().unwrap();
        response = map.get(&req.shorthand).unwrap().1;
        if !response { std::thread::sleep(Duration::from_millis(10)); }
    }
    response
}