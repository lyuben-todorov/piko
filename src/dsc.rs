use std::sync::{Arc, RwLock, mpsc};
use crate::state::{Mode, State, Node};
use std::net::{TcpStream, SocketAddr};

use crate::proto::{Type, ProtoParcel, Body};
use std::io::{Write, Read};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use rayon::prelude::*;

// Start discovery routine
pub fn dsc(state: Arc<RwLock<State>>, neighbour_list: &Vec<SocketAddr>) {

    // begin parallel scope
    let (sender, receiver): (Sender<Vec<Node>>, Receiver<Vec<Node>>) = mpsc::channel();

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

    for node in neighbours {
        println!("{}", node.name)
    }
    state.write().unwrap().change_mode(Mode::WRK);
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

    tcp_stream.write(buffer);

    let mut res: Vec<u8> = Vec::new();

    tcp_stream.read_to_end(&mut res);

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

