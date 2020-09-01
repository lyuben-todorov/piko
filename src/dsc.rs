use std::sync::{Arc, RwLock, mpsc};
use crate::state::{Mode, State, Node};
use std::net::{TcpStream, SocketAddr};

use crate::proto::{Type, ProtoParcel, Body};
use std::io::{Write, Read};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use rayon::prelude::*;
use byteorder::{WriteBytesExt, BigEndian};

// Start discovery routine
pub fn dsc(state: Arc<RwLock<State>>, neighbour_list: &Vec<SocketAddr>) {

    // begin parallel scope
    let (sender, receiver): (Sender<Vec<Node>>, Receiver<Vec<Node>>) = mpsc::channel();

    println!("Attempting to connect to {} hosts", neighbour_list.len());

    // Skip discovery
    if neighbour_list.len() == 0 {
        let mut state = state.write().unwrap();
        state.change_mode(Mode::WRK);
        return;
    }

    let neighbour_list = neighbour_list.as_slice();
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        let state_ref = state.clone();
        discover(&addr, state_ref, s);
    });

    // end parallel scope


    let mut neighbours: HashSet<Node> = HashSet::new();

    for nodes in receiver.iter() {
        neighbours.extend(nodes);
    }

    for node in &neighbours {
        println!("{}", node.name)
    }

    let mut state = state.write().unwrap();
    state.change_mode(Mode::WRK);
    state.cluster_size = neighbours.len();
}

// Request/response on same tcp stream
// Writes result to state after acquiring write lock

fn discover(host: &SocketAddr, state_ref: Arc<RwLock<State>>, tx: &mut Sender<Vec<Node>>) {
    println!("Connecting to {}", host);
    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            return;
        }
    };

    let state = state_ref.read().unwrap();

    let req = ProtoParcel::dsq_req(&state.self_node_information);
    let buffer = serde_cbor::to_vec(&req).unwrap();
    let buffer = buffer.as_slice();

    println!("Buffer size is {} ", buffer.len());

    tcp_stream.write_u8(buffer.len() as u8);
    tcp_stream.write_all(buffer).unwrap();

    let mut res: Vec<u8> = Vec::new();

    tcp_stream.read(&mut res);

    let res: ProtoParcel = serde_cbor::from_slice(res.as_slice()).unwrap();
    match res.parcel_type {
        Type::DscRes => {
            if let Body::DscRes { neighbours } = res.body {
                tx.send(neighbours);
            } else {
                println!("Body-header type mismatch!");
                return;
            }
        }

        Type::ProtoError => {}
        _ => {
            println!("Unexpected response type to discovery request, {}", res.parcel_type);
            return;
        }
    }
}

