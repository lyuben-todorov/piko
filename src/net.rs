use std::net::{TcpListener, TcpStream};

use crate::state::{State};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::Read;
use crate::proto::{ProtoParcel, Type, Body};
use byteorder::{ReadBytesExt, BigEndian};
use std::error::Error;

pub fn read_parcel(stream: &mut TcpStream) -> ProtoParcel {
    let count = stream.read_u8().unwrap();

    println!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];
    stream.read_exact(&mut buf);

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice()).unwrap();
    proto_parcel
}

pub fn listener_thread(_recv: Receiver<u32>, _state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let parcel = read_parcel(&mut stream);

        match parcel.parcel_type {
            Type::DscReq => {
                if let Body::DscReq { identity } = parcel.body {
                    println!("Received DscReq from node {}", identity.name);
                } else {
                    println!("Body-header type mismatch!");
                    return;
                }
            }

            Type::ProtoError => {
                println!("Proto Error")
            }

            _ => {
                println!("Unexpected response type to discovery request, {}", parcel.parcel_type);
                return;
            }
        }
    }
}
