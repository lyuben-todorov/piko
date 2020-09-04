use crate::state::{State, Node, Mode};
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use rayon::prelude::*;
use std::net::{SocketAddr, TcpStream};
use crate::proto::{ProtoParcel, Type, Body};
use crate::net::{write_parcel, read_parcel};

pub fn seq_recovery(state: Arc<RwLock<State>>, neighbour_list: &Vec<Node>) {
    let (sender, receiver): (Sender<u8>, Receiver<u8>) = mpsc::channel(); // return results on channel
    // begin parallel scope
    let neighbour_list: Vec<SocketAddr> = neighbour_list.iter().map(|node| { node.host }).collect();

    let state_ref = state.read().unwrap();
    let id = state_ref.self_node_information.id;
    drop(state_ref);
    let req = ProtoParcel::seq_req(id);

    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        let state_ref = state.clone();
        recover(&addr, &req, s);
    });
    // end parallel scope

    let max_seq = receiver.iter().max_by_key(|seq| *seq).unwrap();
    let mut state = state.write().unwrap();
    state.sequence = max_seq;
    state.change_mode(Mode::Wrk);
}

fn recover(host: &SocketAddr, req_parcel: &ProtoParcel, tx: &mut Sender<u8>) {
    println!("Connecting to {}", host);

    let mut tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            return;
        }
    };
    write_parcel(&mut tcp_stream, &req_parcel);
    let res_parcel = read_parcel(&mut tcp_stream);
    match res_parcel.parcel_type {
        Type::SeqRes => {
            if let Body::SeqRes { seq_number } = res_parcel.body {
                tx.send(seq_number).unwrap();
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