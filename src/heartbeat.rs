use crate::state::{State, Mode, Node};
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::net::{SocketAddr, TcpStream};
use crate::proto::{ProtoParcel, Type};
use rayon::prelude::*;
use crate::net::{read_parcel, write_parcel};
use clokwerk::{Scheduler, TimeUnits};
use std::time::Duration;
use crate::internal::ThreadSignal;


pub fn heartbeat(state: Arc<RwLock<State>>, heart_rate: u32, _timeout: u32, rx: Receiver<ThreadSignal>) {
    let mut scheduler = Scheduler::new();

    scheduler.every(heart_rate.seconds()).run(move || {
        let state_ref = state.read().unwrap();

        if state_ref.self_node_information.mode == Mode::Wrk {
            let (sender, receiver): (Sender<bool>, Receiver<bool>) = mpsc::channel(); // setup channel for results

            let neighbour_list: Vec<Node> = state_ref.neighbours.values().cloned().collect();

            let neighbour_list: Vec<SocketAddr> = neighbour_list.iter().map(|node| { node.host }).collect(); // get socketaddrs

            let _id = state_ref.self_node_information.id;
            drop(state_ref); // drop lock

            let req = ProtoParcel::ping();

            // begin parallel scope
            neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
                ping(&addr, &req, s);
            });
            // end parallel scope

            for _result in receiver.iter() {
            }
        } else {
            return;
        }
    });

    let thread_handle = scheduler.watch_thread(Duration::from_millis(100));

    println!("Started heartbeat thread!");

    for sig in rx.iter() {
        match sig {
            ThreadSignal::StopProcess => {
                thread_handle.stop();
                println!("Stopping heartbeat thread!");
                return;
            }
            _ => {
                println!("Unknown signal sent to monitor thread")
            }
        }
    }
}

fn ping(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<bool>) {
    println!("Sending heartbeat to {}", host);

    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            tx.send(false).unwrap();
            return;
        }
    };
    write_parcel(&mut tcp_stream, &req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);
    match res_parcel.parcel_type {
        Type::Pong => {
            tx.send(true).unwrap();
        }
        Type::ProtoError => {}
        _ => {
            println!("Unexpected response type to Ping, {}", res_parcel.parcel_type);
            tx.send(false).unwrap();
            return;
        }
    }
}