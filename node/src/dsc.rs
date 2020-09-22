use std::sync::{Arc, RwLock, mpsc};
use crate::state::{Mode, State, Node};
use std::net::{TcpStream, SocketAddr};

use crate::proto::{Type, ProtoParcel, Body};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::HashSet;
use rayon::prelude::*;
use log::{debug, error, info, trace, warn};

use crate::net::{write_parcel, read_parcel};

// Start discovery routine
pub fn dsc(state: Arc<RwLock<State>>, neighbour_list: &Vec<SocketAddr>) {
    // Skip discovery
    if neighbour_list.len() == 0 {
        let mut state = state.write().unwrap();
        state.change_mode(Mode::Wrk);
        return;
    }

    info!("Attempting to connect to {} hosts", neighbour_list.len());

    let (sender, receiver): (Sender<Vec<Node>>, Receiver<Vec<Node>>) = mpsc::channel(); // make channel for responses

    let state_ref = state.read().unwrap();
    let node = state_ref.self_node_information.clone();
    drop(state_ref);
    let req_parcel = ProtoParcel::dsc_req(node); // Use same object for serializing each request

    // begin parallel scope
    let neighbour_list = neighbour_list.as_slice();
    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        let _state_ref = state.clone();
        discover(&addr, &req_parcel, s);
    });
    // end parallel scope

    let mut neighbours: HashSet<Node> = HashSet::new();

    // collect results
    for nodes in receiver.iter() {
        neighbours.extend(nodes);
    }
    let mut state = state.write().unwrap(); // acquire write lock
    for neighbour in neighbours {
        info!("Found {}:{}!", neighbour.name, neighbour.mode);
        state.add_neighbour(neighbour);
    }
    state.change_mode(Mode::Wrk);
}

// Request/response on same tcp stream
// Writes result to state after acquiring write lock
fn discover(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<Vec<Node>>) {
    info!("Connecting to {}", host);
    let mut stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            error!("{}: {}", err, host);
            return;
        }
    };

    write_parcel(&mut stream, req_parcel);

    let res_parcel = match read_parcel(&mut stream) {
        Ok(parcel) => parcel,
        Err(e) => {
            error!("Invalid parcel! {}", e);
            return;
        }
    };

    match res_parcel.parcel_type {
        Type::DscRes => {
            if let Body::DscRes { neighbours } = res_parcel.body {
                tx.send(neighbours).unwrap();
            } else {
                error!("Body-header type mismatch!");
                return;
            }
        }

        Type::ProtoError => {}
        _ => {
            error!("Unexpected response type to discovery request, {}", res_parcel.parcel_type);
            return;
        }
    }
}

