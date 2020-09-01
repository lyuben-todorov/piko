use std::net::{TcpListener};

use crate::state::{State};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

pub fn listener_thread(rx: Receiver<u32>, tx: Sender<u32>, state: Arc<RwLock<State>>, socket: TcpListener) {
    for stream in socket.incoming() {
        stream.unwrap();

    }

}
