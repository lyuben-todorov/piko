use std::net::{TcpListener, TcpStream};

use crate::state::{State, Node};

use std::sync::mpsc::{Receiver};
use std::sync::{Arc, RwLock};
use std::io::{Read, Write};
use crate::proto::{ProtoParcel, Type, Body};
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::internal::ThreadSignal;


pub fn read_parcel(stream: &mut TcpStream) -> ProtoParcel {
    let count = stream.read_u8().unwrap();

    // println!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];
    stream.read_exact(&mut buf).unwrap();

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice()).unwrap();
    proto_parcel
}

pub fn write_parcel(stream: &mut TcpStream, parcel: &ProtoParcel) {
    let parcel = serde_cbor::to_vec(&parcel).unwrap();
    let buf = parcel.as_slice();
    let count = buf.len();

    // println!("Writing {} bytes", count);
    stream.write_u8(count as u8).unwrap();
    stream.write_all(buf).unwrap();
}

pub fn ack(response: ProtoParcel, ack_id: u64) -> ThreadSignal {
    match response.parcel_type {
        Type::Ack => {
            if let Body::Ack { message_id } = response.body {
                if message_id == ack_id {
                    println!("Acked {}", message_id);
                    ThreadSignal::Success
                } else {
                    ThreadSignal::Fail
                }
            } else {
                println!("Body-header type mismatch!");
                ThreadSignal::Fail
            }
        }
        Type::ProtoError => {
            ThreadSignal::Fail
        }
        _ => {
            println!("Expected acknowledge, got {}", response.parcel_type);
            ThreadSignal::Fail
        }
    }
}

pub fn listener_thread(_recv: Receiver<ThreadSignal>, state: Arc<RwLock<State>>, socket: TcpListener) {
    println!("Started Listener thread!");

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let state_ref = state.clone();

        rayon::spawn(move || {
            let parcel = read_parcel(&mut stream);

            match parcel.parcel_type {
                Type::DscReq => {
                    if let Body::DscReq { identity } = parcel.body {
                        println!("Received DscReq with id {} from node {}", parcel.id, parcel.sender_id);

                        let mut neighbours: Vec<Node> = Vec::new();

                        let mut state = state_ref.write().unwrap(); // acquire write lock
                        let state_neighbours: Vec<Node> = state.neighbours.values().cloned().collect();
                        let self_node = state.self_node_information.clone();

                        println!("Adding {} to state", identity.name);
                        state.add_neighbour(identity); // add node to state after neighbours are cloned
                        drop(state); // drop write lock before tcp writes

                        neighbours.extend_from_slice(state_neighbours.as_slice());
                        neighbours.push(self_node);

                        let parcel = ProtoParcel::dsc_res(neighbours);

                        write_parcel(&mut stream, &parcel);
                    } else {
                        println!("Body-header type mismatch!");
                        return;
                    }
                }

                Type::SeqReq => {
                    println!("Received SeqReq with id {} from node {}", parcel.id, parcel.sender_id);
                    let state = state_ref.write().unwrap(); // acquire write lock
                    let seq = state.sequence;
                    let parcel = ProtoParcel::seq_res(seq);
                    drop(state);
                    write_parcel(&mut stream, &parcel);
                }
                Type::Ping => {
                    println!("Received Ping with id {} from node {}", parcel.id, parcel.sender_id);
                    let state = state_ref.write().unwrap(); // acquire write lock
                    let parcel = ProtoParcel::pong();
                    drop(state);
                    write_parcel(&mut stream, &parcel);
                }
                Type::ProtoError => {
                    println!("Proto Error")
                }
                Type::StateChange => {
                    if let Body::StateChange { mode } = parcel.body {
                        println!("Received StateChange with id {} from node {}", parcel.id, parcel.sender_id);
                        let mut state = state_ref.write().unwrap();
                        state.neighbours.entry(parcel.sender_id).and_modify(|node| {
                            node.mode = mode
                        });
                        drop(state);
                        let parcel = ProtoParcel::ack(parcel.id);
                        write_parcel(&mut stream, &parcel);
                    }
                }
                Type::AddNode => {
                    if let Body::AddNode { nodes } = parcel.body {
                        println!("Received AddNode with id {} from node {}", parcel.id, parcel.sender_id);
                        let mut state = state_ref.write().unwrap();
                        for node in nodes {
                            state.add_neighbour(node);
                        }
                    }
                }
                _ => {
                    println!("Unexpected message type!, {}", parcel.parcel_type);
                    return;
                }
            }
        });
    }
}
