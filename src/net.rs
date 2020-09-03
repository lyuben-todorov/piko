use std::net::{TcpListener, TcpStream};

use crate::state::{State, Node};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};
use crate::proto::{ProtoParcel, Type, Body};
use byteorder::{ReadBytesExt, WriteBytesExt};


pub fn read_parcel(stream: &mut TcpStream) -> ProtoParcel {
    let count = stream.read_u8().unwrap();

    println!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];
    stream.read_exact(&mut buf);

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice()).unwrap();
    proto_parcel
}

pub fn write_parcel(stream: &mut TcpStream, parcel: ProtoParcel) {
    let parcel = serde_cbor::to_vec(&parcel).unwrap();
    let buf = parcel.as_slice();
    let count = buf.len();

    println!("Writing {} bytes", count);
    stream.write_u8(count as u8);
    stream.write_all(buf);
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
                        let mut state = state_ref.write().unwrap();
                        let mut neighbours: Vec<Node> = Vec::new();
                        let state_neighbours: Vec<Node> = state.neighbours.values().cloned().collect();
                        let self_node = state.self_node_information.clone();
                        println!("Adding {} to state", identity.name);
                        state.add_neighbour(identity); // add node to state after neighbours are cloned

                        let id: u16 = self_node.id;
                        neighbours.extend_from_slice(state_neighbours.as_slice());
                        neighbours.push(self_node);
                        let parcel = ProtoParcel::dsc_res(id, neighbours);

                        write_parcel(&mut stream, parcel);
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
