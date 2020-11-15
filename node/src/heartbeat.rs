use crate::state::{State, Mode, Node};
use std::sync::{RwLock, Arc};
use crossbeam_channel::{Sender, Receiver};
use crate::proto::{ProtoParcel, Type};
use crate::net::{read_parcel, write_parcel};
use clokwerk::{Scheduler, TimeUnits};
use std::time::Duration;
use crate::internal::TaskSignal;
use std::collections::{HashMap, HashSet};

use log::{debug, error, info, warn};
use tokio::net::TcpStream;

pub async fn heartbeat(state: Arc<RwLock<State>>, heart_rate: u32, timeout: u32, rx: Receiver<TaskSignal>) {
    let mut scheduler = Scheduler::new();
    // map node id to amount of timeouts
    let mut timeouts: HashMap<u64, u8> = HashMap::new();

    scheduler.every(heart_rate.seconds()).run(move || {
        let state_ref = state.read().unwrap();
        let new_keys: HashSet<u64> = state_ref.get_neighbour_keys();

        // Add new keys
        for key in new_keys {
            if !timeouts.contains_key(&key) {
                info!("Adding {} to monitor", key);
                timeouts.insert(key, 0);
            }
        }

        if state_ref.mode == Mode::Wrk {
            let (sender, receiver): (Sender<(u64, bool)>, Receiver<(u64, bool)>) = crossbeam_channel::unbounded(); // setup channel for results

            let neighbour_list: Vec<Node> = state_ref.get_active_neighbours();
            drop(state_ref); // drop lock

            let req = ProtoParcel::ping();

            // begin parallel scope
            // TODO
            // neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
            //     ping(&addr, &req, s);
            // });
            // // end parallel scope

            for result in receiver.iter() {
                let (id, node_response) = result;

                if !node_response {
                    let entry = timeouts.entry(id).or_insert(0);
                    *entry += 1;
                    warn!("Node with id {} timed out for {}", id, entry);
                    if *entry > timeout as u8 {
                        error!("Node with id {} timed out for more than {} heartbeats. ", id, timeout);
                        let mut state_ref = state.write().unwrap();
                        let node = state_ref.neighbours.entry(id);
                        node.and_modify(|x| { x.mode = Mode::TimedOut });
                    }
                } else {
                    timeouts.insert(id, 0);
                }
            }
        } else {
            return;
        }
    });

    let thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    info!("Started heartbeat thread!");

    for sig in rx.iter() {
        match sig {
            TaskSignal::StopProcess => {
                thread_handle.stop();
                info!("Stopping heartbeat thread!");
                return;
            }
            _ => {
                error!("Unknown signal sent to monitor thread")
            }
        }
    }
}

async fn ping(node: &Node, req_parcel: &ProtoParcel, tx: &mut Sender<(u64, bool)>) {
    // info!("Sending Ping to {}", node.id);

    let mut stream = match TcpStream::connect(node.external_addr).await {
        Ok(stream) => stream,
        Err(err) => {
            debug!("{}: {}", err, node.id);
            tx.send((node.id, false)).unwrap();
            return;
        }
    };
    write_parcel(&mut stream, &req_parcel).await;
    let res_parcel = match read_parcel(&mut stream).await {
        Ok(parcel) => parcel,
        Err(e) => {
            error!("Invalid parcel! {}", e);
            tx.send((node.id, false)).unwrap();
            return;
        }
    };
    match res_parcel.parcel_type {
        Type::Pong => {
            tx.send((node.id, true)).unwrap();
        }
        Type::ProtoError => {}
        _ => {
            error!("Unexpected response type to Ping, {}", res_parcel.parcel_type);
            tx.send((node.id, false)).unwrap();
            return;
        }
    }
}