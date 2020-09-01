use std::net::{TcpListener};

use crate::state::{State};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::proto::{ProtoParcel};

pub fn listener_thread(_recv: Receiver<u32>, _state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {

        let mut stream = stream.unwrap();
        println!("{}", stream.peer_addr().unwrap());

        let mut bytes: Vec<u8> = Vec::new();
        println!("bytes are {}", String::from_utf8_lossy(bytes.as_slice()));

        let count = stream.read_to_end(&mut bytes).unwrap();
        println!("count is {}", count);

        let _parcel: ProtoParcel = serde_cbor::from_slice(bytes.as_slice()).unwrap();


    }
}
