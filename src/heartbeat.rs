use crate::state::{State, Mode, Node};
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::net::{TcpStream};
use crate::proto::{ProtoParcel, Type};
use rayon::prelude::*;
use crate::net::{read_parcel, write_parcel};
use clokwerk::{Scheduler, TimeUnits};
use std::time::Duration;
use crate::internal::ThreadSignal;
use std::collections::{HashMap, HashSet};

use log::{debug, error, info, trace, warn};

pub fn heartbeat(state: Arc<RwLock<State>>, heart_rate: u32, _timeout: u32, rx: Receiver<ThreadSignal>) {
    let mut scheduler = Scheduler::new();
    // map node id to amount of timeouts
    let mut timeouts: HashMap<u16, u8> = HashMap::new();

    scheduler.every(heart_rate.seconds()).run(move || {
        let state_ref = state.read().unwrap();
        let new_keys: HashSet<u16> = state_ref.get_neighbour_keys();

        // Add new keys
        for key in new_keys {
            if !timeouts.contains_key(&key) {
                info!("Adding {} to monitor", key);
                timeouts.insert(key, 0);
            }
        }

        if state_ref.self_node_information.mode == Mode::Wrk {
            let (sender, receiver): (Sender<(u16, bool)>, Receiver<(u16, bool)>) = mpsc::channel(); // setup channel for results

            let neighbour_list: Vec<Node> = state_ref.get_active_neighbours();
            drop(state_ref); // drop lock

            let req = ProtoParcel::ping();

            // begin parallel scope
            neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
                ping(&addr, &req, s);
            });
            // end parallel scope

            for result in receiver.iter() {
                if !result.1 {
                    let entry = timeouts.entry(result.0).or_insert(0);
                    *entry += 1;
                    if *entry > 5 {
                        warn!("Node with id {} timed out for {}", result.0, entry);
                    }
                } else {
                    timeouts.insert(result.0, 0);
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
            ThreadSignal::StopProcess => {
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

fn ping(node: &Node, req_parcel: &ProtoParcel, tx: &mut Sender<(u16, bool)>) {
    info!("Sending Ping to {}", node.id);

    let mut tcp_stream = match TcpStream::connect(node.host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, node.id);
            tx.send((node.id, false)).unwrap();
            return;
        }
    };
    write_parcel(&mut tcp_stream, &req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);
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