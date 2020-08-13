use std::sync::{Mutex, Arc};
use crate::state::{Mode, State, Node};
use std::collections::HashMap;
use crate::net::DscConnection;
use std::net::TcpStream;
use crate::state::Mode::DSC;

// Start discovery routine
pub fn dsc(state: &mut State, neighbour_list: &[String]) {
    let state = Arc::new(Mutex::new(state));

    // begin parallel scope
    rayon::scope(|s| {
        for host in neighbour_list {
            let state = Arc::clone(&state);
            s.spawn(move |_| handshake(&host, state));
        }
    });
    // end parallel scope

    let mut state = state.lock().unwrap();
    state.change_mode(Mode::WRK);
}

fn handshake(host: &String, state: Arc<Mutex<&mut State>>) {
    println!("{}", host);
    let tcp_stream = match TcpStream::connect(host) {
        Ok(res) => res,
        Err(e) => {
            return;
        }
    };

    let conn = DscConnection::new(tcp_stream);

    let mut state = state.lock().unwrap();
    state.add_neighbour("", Node::new(String::new(), DSC, String::new()));
}