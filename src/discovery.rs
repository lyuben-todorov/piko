use threadpool::ThreadPool;
use std::sync::{Mutex, Arc};
use crate::state::Mode::{SHUTDOWN, DSC};
use crate::state::{Mode, State, Node};
use std::thread::JoinHandle;
use std::thread;
use std::collections::HashMap;

// Start discovery routine
pub fn dsc(thread_pool: &ThreadPool, state: &mut State, neighbour_list:  &Vec<String>) {
    let node_list: Arc<Mutex<HashMap<String, Node>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for host in neighbour_list {
        let nodes = Arc::clone(&node_list);
        thread_pool.execute(move || handshake(&host, nodes));
    }

    thread_pool.join();
    state.change_mode(Mode::WRK);
}

fn handshake(host: &String, node_list: Arc<Mutex<HashMap<String, Node>>>) {
    // do requests
}