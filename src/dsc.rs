use std::sync::{Arc, RwLock, mpsc};
use crate::state::{Mode, State, Node};
use std::net::{TcpStream, SocketAddr};

use crate::proto::{Type, ProtoParcel, Body};
use std::io::{Write, Read};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use rayon::prelude::*;

use crate::net::{write_parcel, read_parcel};

// Start discovery routine
pub fn dsc(state: Arc<RwLock<State>>, neighbour_list: &Vec<SocketAddr>) {
    // Skip discovery
    if neighbour_list.len() == 0 {
        let mut state = state.write().unwrap();
        state.change_mode(Mode::Wrk);
        return;
    }

    println!("Attempting to connect to {} hosts", neighbour_list.len());

    let (sender, receiver): (Sender<Vec<Node>>, Receiver<Vec<Node>>) = mpsc::channel(); // return results on channel
    // begin parallel scope
    let neighbour_list = neighbour_list.as_slice();
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        let state_ref = state.clone();
        discover(&addr, state_ref, s);
    });
    // end parallel scope

    let mut neighbours: HashSet<Node> = HashSet::new();

    // collect results
    for nodes in receiver.iter() {
        neighbours.extend(nodes);
    }
    let mut state = state.write().unwrap(); // acquire write lock
    state.change_mode(Mode::Wrk);
    for neighbour in neighbours {
        println!("Found {}!", neighbour.name);
        state.add_neighbour(neighbour);
    }
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

    let self_node = state.self_node_information.clone();
    let req_parcel = ProtoParcel::dsc_req(self_node);

    write_parcel(&mut tcp_stream, req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);

    match res_parcel.parcel_type {
        Type::DscRes => {
            if let Body::DscRes { neighbours } = res_parcel.body {
                tx.send(neighbours).unwrap();
            } else {
                println!("Body-header type mismatch!");
                return;
            }
        }

        Type::ProtoError => {}
        _ => {
            println!("Unexpected response type to discovery request, {}", res_parcel.parcel_type);
            return;
        }
    }
}

