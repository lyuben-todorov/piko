use std::net::{TcpListener};

use crate::state::{State};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::proto::{ProtoError, ProtoParcel};

pub fn listener_thread(rx: Receiver<u32>, tx: Sender<u32>, state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {
        println!("ping");
        let mut stream = stream.unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        stream.read_to_end(&mut bytes);

        let parcel: ProtoParcel = serde_cbor::from_slice(bytes.as_slice()).unwrap();

        println!("{}", parcel.id);

    }
}
