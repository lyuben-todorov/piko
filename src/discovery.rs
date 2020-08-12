use std::sync::{Mutex, Arc};
use crate::state::{Mode, State, Node};
use std::collections::HashMap;

// Start discovery routine
pub fn dsc(state: &mut State, neighbour_list: &[String]) {
    let node_list: Arc<Mutex<HashMap<String, Node>>> =
        Arc::new(Mutex::new(HashMap::new()));

    rayon::scope(|s| {
        for host in neighbour_list {
            let list = Arc::clone(&node_list);
            s.spawn(move |_| handshake(&host, list));
        }
    });

    state.change_mode(Mode::SHUTDOWN);
}

fn handshake(host: &String, node_list: Arc<Mutex<HashMap<String, Node>>>) {
    println!("{}", host);
}