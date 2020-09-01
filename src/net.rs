use std::net::{TcpListener};

use crate::state::{State};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::proto::{ProtoError, ProtoParcel};

pub fn listener_thread(recv: Receiver<u32>, state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {

        let mut stream = stream.unwrap();
        println!("{}", stream.peer_addr().unwrap());

        let mut bytes: Vec<u8> = Vec::new();
        println!("bytes are{}", String::from_utf8_lossy(bytes.as_slice()));

        let count = stream.read_to_end(&mut bytes).unwrap();
        println!("count is {}", count);

        let parcel: ProtoParcel = serde_cbor::from_slice(bytes.as_slice()).unwrap();


    }
}
