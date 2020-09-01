use std::net::{TcpListener, TcpStream};

use crate::state::{State};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::proto::{ProtoParcel};
use byteorder::{ReadBytesExt, BigEndian};
use std::error::Error;

pub fn read_parcel(stream: &mut TcpStream) -> ProtoParcel {
    let count = stream.read_u8().unwrap();

    println!("Expecting {} bytes", count);

    let mut bytes: Vec<u8> = Vec::with_capacity(count as usize);
    stream.read_exact(&mut *bytes).unwrap();

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(bytes.as_slice()).unwrap();
    proto_parcel
}

pub fn listener_thread(_recv: Receiver<u32>, _state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let parcel = read_parcel(&mut stream);

        println!("{} {}", parcel.id, parcel.parcel_type);
    }
}
