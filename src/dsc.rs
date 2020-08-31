use std::sync::{Arc, Mutex};
use crate::state::{Mode, State};
use crate::net::handshake;
use std::net::TcpStream;
use std::alloc::System;

// Start discovery routine
pub fn dsc(state: Arc<Mutex<State>>, neighbour_list: &Vec<String>) {
    // begin parallel scope
    rayon::scope(|s| {
        for host in neighbour_list {
            let state = state.clone();
            s.spawn(move |_| discover(&host, state));
        }
    });

    // end parallel scope

    let mut state = state.lock().unwrap();

    for node in state.neighbours.values() {
        println!("{}", node.name);
    }

    state.change_mode(Mode::WRK);
}

fn discover(host: &String, state: Arc<Mutex<State>>) {
    println!("{}", host);
    let tcp_stream = TcpStream::connect(host)?;
    let node = handshake(host, &state.clone().self_node).unwrap();
}