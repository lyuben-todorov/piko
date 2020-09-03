use std::net::{TcpListener, TcpStream};

use crate::state::{State, Node};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};
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

pub fn listener_thread(_recv: Receiver<u32>, state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let state_ref = state.clone();

        rayon::spawn(move || {
            let parcel = read_parcel(&mut stream);

            match parcel.parcel_type {
                Type::DscReq => {
                    if let Body::DscReq { identity } = parcel.body {
                        println!("Received DscReq from node {}", identity.name);
                        let mut neighbours: Vec<Node> = Vec::new();
                        let state = state_ref.read().unwrap();
                        let state_neighbours: Vec<Node> = state.neighbours.values().cloned().collect();
                        let self_node = state.self_node_information.clone();
                        neighbours.extend_from_slice(state_neighbours.as_slice());
                        let parcel = ProtoParcel::dsc_res(self_node, neighbours);
                        let parcel = serde_cbor::to_vec(&parcel).unwrap();
                        stream.write_all(parcel.as_slice());
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
        });
    }
}
