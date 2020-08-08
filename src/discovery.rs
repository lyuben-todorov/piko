use threadpool::ThreadPool;
use std::sync::{Mutex, Arc};
use crate::state::Mode::{SHUTDOWN, DSC};
use crate::state::{Mode, State};

// Start discovery routine
pub fn dsc(thread_pool: &ThreadPool, state: &mut State) {
    let neighbouring_hosts = &state.neighbours;
    


    thread_pool.join();
    state.change_mode(Mode::SHUTDOWN);
}

fn handshake(String: host) {}