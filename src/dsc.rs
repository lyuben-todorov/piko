use std::sync::{Mutex, Arc};
use crate::state::{Mode, State, Node};
use std::collections::HashMap;
use crate::net::DscConnection;
use std::net::TcpStream;

// Start discovery routine
pub fn dsc(state: &mut State, neighbour_list: &[String]) {
    let node_list: Arc<Mutex<HashMap<String, Node>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // begin parallel scope
    rayon::scope(|s| {
        for host in neighbour_list {
            let list = Arc::clone(&node_list);
            s.spawn(move |_| handshake(&host, list));
        }
    });
    // end parallel scope

    let node_list = node_list.lock().unwrap();

    for (string, node) in node_list.iter() {
        state.add_neighbour(string, node.clone());
    }

    state.change_mode(Mode::WRK);
}

fn handshake(host: &String, node_list: Arc<Mutex<HashMap<String, Node>>>) {
    println!("{}", host);
    let tcp_stream = match TcpStream::connect(host) {
        Ok(res) => res,
        Err(e) => {
            return;
        }
    };

    let conn = DscConnection::new(tcp_stream);
}