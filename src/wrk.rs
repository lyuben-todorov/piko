use std::net::TcpListener;
use crate::state::State;

pub fn wrk(state: &mut State, listener: TcpListener) {
    if state.cluster_size == 0 {
        println!("Operating in single node cluster.")
        // don't spawn heartbeat thread
    } else {
        println!("Spawning heartbeat thread.")
    }

    // main listener loop
    for stream in listener.incoming() {}
}