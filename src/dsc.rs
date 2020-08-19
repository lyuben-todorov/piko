use std::sync::{Mutex, Arc};
use crate::state::{Mode, State, Node};
use std::collections::HashMap;
use crate::net::DscConnection;
use std::net::TcpStream;
use crate::state::Mode::DSC;
use std::ops::Deref;

// Start discovery routine
pub fn dsc(state: &mut State, neighbour_list: &[String]) {
    let immutable_state: Arc<&State> = Arc::new((state));

    // begin parallel scope
    rayon::scope(|s| {
        for host in neighbour_list {
            let immutable_state = immutable_state.clone();
            s.spawn(move |_| handshake(&host, immutable_state));
        }
    });

    // end parallel scope

    state.change_mode(Mode::WRK);
}

fn handshake(host: &String, state: Arc<&State>) {
    println!("{}", host);
    let tcp_stream = match TcpStream::connect(host) {
        Ok(res) => res,
        Err(e) => {
            return;
        }
    };

    let mut dsc_conn = DscConnection::new(tcp_stream, *state);

    dsc_conn.handshake();
}