use threadpool::ThreadPool;
use crate::State;
use std::sync::{Mutex, Arc};
use crate::State::{SHUTDOWN, DSC};

pub fn dsc(threadpool: &ThreadPool, mut state: &State) {
    threadpool.execute(|| println!("Hi"));

    match state {
        DSC => {
            state = &SHUTDOWN;
            print!("Yes");
        }
        _ => {}
    }
}