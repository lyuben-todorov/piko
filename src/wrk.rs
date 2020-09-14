use crate::state::{State, Mode};
use std::sync::{RwLock, Arc};
use std::sync::mpsc::{Receiver};
use std::thread::park;
use crate::req::{push_state::push_state, seq_recovery::seq_recovery};


use log::{debug, error, info, trace, warn};
use std::collections::{BinaryHeap};
use chrono::{DateTime, Utc};
use crate::proto::MessageWrapper;
use enum_dispatch::enum_dispatch;

use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ResourceRequest {
    owner: u16,
    message_hash:u64,
    timestamp: DateTime<Utc>,
    sequence: u16,
}

impl Ord for ResourceRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

impl PartialOrd for ResourceRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ResourceRelease {
    owner: u16,
    message_hash:u64,
    timestamp: DateTime<Utc>,
    message: MessageWrapper,
    local: bool,
    sequence: u16,
}

#[enum_dispatch]
pub enum Pledge {
    ResourceRequest(ResourceRequest),
    ResourceRelease(ResourceRelease),
}


///
/// Tasked with maintaining protocol consistency
///
pub fn wrk(state: Arc<RwLock<State>>, recevier: &Receiver<Pledge>) {
    let mut state_ref = state.write().unwrap();

    let _local_id = state_ref.self_node_information.id;

    let mut pledgeQueue: BinaryHeap<ResourceRequest> = BinaryHeap::new();

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

    let _clock = Utc::now();

    for pledge in recevier.iter() {
        match pledge {
            ///
            /// Upon resource request, the node pushes the request to the priority queue.
            ///
            Pledge::ResourceRequest(req) => {
                    pledgeQueue.push(req);
            }
            ///
            /// Upon resource release the node checks the owner of the next ResourceRequest.
            /// If it is the owner, it sends a Commit request, asking its neighbours to confirm the
            /// commit lock. After receiving all acks the nodes sends it's ResourceRelease together
            /// with the message and pops its message queue.
            ///
            Pledge::ResourceRelease(rel) => {
                let req = pledgeQueue.peek().unwrap();
                if req.message_hash == rel.message_hash && rel.timestamp > req.timestamp {
                    // release resource
                } else {

                }
            }
        }
    }


    park();
}

