use std::net::TcpListener;
use crate::state::State;

pub fn wrk(state: &mut State, listener: &TcpListener) {
    if state.cluster_size == 0 {
        println!("Operating in single node cluster.");
        // don't spawn heartbeat thread
    } else {
        println!("Spawning heartbeat thread.");
        spawn_heartbeat();
    }


    // main listener loop
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
    }
}

fn spawn_heartbeat() {}