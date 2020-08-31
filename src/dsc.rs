use std::sync::{Arc, Mutex, RwLock};
use crate::state::{Mode, State};
use std::net::TcpStream;
use std::alloc::System;
use crate::proto::ProtoParcel;
use std::io::{Write, Read};

// Start discovery routine
pub fn dsc(state: Arc<RwLock<State>>, neighbour_list: &Vec<String>) {
    // begin parallel scope
    rayon::scope(|s| {
        for host in neighbour_list {
            s.spawn(move |_| discover(&host, state.clone()));
        }
    });

    // end parallel scope

    let mut state = state.lock().unwrap();

    for node in state.neighbours.values() {
        println!("{}", node.name);
    }

    state.change_mode(Mode::WRK);
}

// Request/response on same tcp stream
// Writes result to state after acquiring write lock
fn discover(host: &String, state: Arc<RwLock<State>>) {
    println!("{}", host);
    let mut tcp_stream = TcpStream::connect(host).unwrap();

    let req = ProtoParcel::dsq_req(&state.read().unwrap().self_node);
    let buffer = serde_cbor::to_vec(&req).unwrap().as_slice();

    tcp_stream.write(buffer);

    let mut res: Vec<u8> = Vec::new();
    tcp_stream.read_to_end(&mut res);

    let res: ProtoParcel = serde_cbor::from_slice(res.as_slice()).unwrap();
    match res.parcel_type {

    }
}

