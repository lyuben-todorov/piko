use std::net::{TcpListener};

use crate::state::{State};




use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

pub fn listener_thread(_rx: Receiver<u32>, _tx: Sender<u32>, _state: Arc<RwLock<State>>, _socket: TcpListener) {}
