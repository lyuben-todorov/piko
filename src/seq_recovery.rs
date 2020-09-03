use crate::state::{State, Node};
use std::sync::{RwLock, Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use rayon::prelude::*;
use std::net::{SocketAddr, TcpStream};
use crate::proto::ProtoParcel;

pub fn seq_recovery(state: Arc<RwLock<State>>, neighbour_list: &Vec<Node>) {
    let (sender, _receiver): (Sender<u8>, Receiver<u8>) = mpsc::channel(); // return results on channel
    // begin parallel scope
    let neighbour_list: Vec<SocketAddr> = neighbour_list.iter().map(|node|{node.host}).collect();

    neighbour_list.into_par_iter().for_each_with(sender, |s, addr| {
        let state_ref = state.clone();
        recover(&addr, state_ref, s);
    });
    // end parallel scope
}

fn recover(host: &SocketAddr, state_ref: Arc<RwLock<State>>, _tx: &mut Sender<u8>) {
    println!("Connecting to {}", host);
    let _tcp_stream = match TcpStream::connect(host) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}: {}", err, host);
            return;
        }
    };
    let state = state_ref.read().unwrap();

    let _req = ProtoParcel::seq_req(state.self_node_information.id);
}