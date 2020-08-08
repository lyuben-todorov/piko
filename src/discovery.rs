use threadpool::ThreadPool;
use crate::State;
use std::sync::{Mutex, Arc};
use crate::State::{SHUTDOWN, DSC};

pub fn dsc(thread_pool: &ThreadPool, state: &State) -> State {
    thread_pool.execute(|| {});

    match state {
        State::DSC => {
            State::SHUTDOWN
        }
        _ => { State::SHUTDOWN }
    }
}