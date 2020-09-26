use std::net::{TcpListener, TcpStream};

use crate::state::{State, Node};


use std::sync::{Arc, RwLock, Mutex};
use std::io::{Read, Write};
use crate::proto::{ProtoParcel, Type, Body, MessageWrapper, ResourceRequest, ResourceRelease, Pledge};
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::internal::ThreadSignal;
use crate::req::add_node::add_node;

use log::{debug, error, info, trace, warn};
use std::collections::{BinaryHeap, HashMap};
use sha2::{Digest};
use std::error::Error;
use std::sync::mpsc::Sender;


pub fn read_parcel(stream: &mut TcpStream) -> Result<ProtoParcel, Box<dyn Error>> {
    let size = stream.read_u8()?;

    // debug!("Expecting {} bytes", size);

    let mut buf = vec![0u8; size as usize];
    stream.read_exact(&mut buf)?;

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice())?;
    Ok(proto_parcel)
}

pub fn write_parcel(stream: &mut TcpStream, parcel: &ProtoParcel) {
    let parcel = serde_cbor::to_vec(&parcel).unwrap();
    let buf = parcel.as_slice();
    let count = buf.len();

    // println!("Writing {} bytes", count);
    stream.write_u8(count as u8).unwrap();
    stream.write_all(buf).unwrap();
}

pub fn is_acked(response: ProtoParcel, ack_id: u64) -> ThreadSignal {
    match response.parcel_type {
        Type::Ack => {
            if let Body::Ack { message_id } = response.body {
                if message_id == ack_id {
                    info!("Acked {}", message_id);
                    ThreadSignal::Success
                } else {
                    ThreadSignal::Fail
                }
            } else {
                error!("Body-header type mismatch!");
                ThreadSignal::Fail
            }
        }
        Type::ProtoError => {
            ThreadSignal::Fail
        }
        _ => {
            error!("Expected acknowledge, got {}", response.parcel_type);
            ThreadSignal::Fail
        }
    }
}

pub fn listener_thread(socket: TcpListener, state: Arc<RwLock<State>>, pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
                       f_access: Arc<Mutex<bool>>, _wrk: Sender<Pledge>) {
    info!("Started Listener thread!");

    let _pending_messages: Arc<HashMap<u16, MessageWrapper>> = Arc::new(HashMap::new());

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let state_ref = Arc::clone(&state);
        let f_access = Arc::clone(&f_access);
        let pledge_queue = Arc::clone(&pledge_queue);
        rayon::spawn(move || {
            let parcel = match read_parcel(&mut stream) {
                Ok(parcel) => parcel,
                Err(e) => {
                    error!("Invalid parcel! {}", e);
                    return;
                }
            };

            match parcel.parcel_type {
                Type::DscReq => {
                    if let Body::DscReq { identity } = parcel.body {
                        info!("Received DscReq with id {} from node {}", parcel.id, parcel.sender_id);

                        let mut neighbours: Vec<Node> = Vec::new();

                        let mut state_ref = state_ref.write().unwrap(); // acquire write lock
                        let state_neighbours: Vec<Node> = state_ref.neighbours.values().cloned().collect();

                        let self_node = state_ref.get_node_information();

                        // Push found node to neighbours
                        let mut update: Vec<Node> = Vec::new();
                        update.push(identity.clone());
                        info!("Pushing new node to neighbours!");
                        add_node(&state_ref.get_neighbour_addrs(), update);

                        info!("Adding {} to state", identity.name);
                        state_ref.add_neighbour(identity); // add node to state after neighbours are cloned
                        drop(state_ref); // drop write lock before tcp writes

                        neighbours.extend_from_slice(state_neighbours.as_slice());
                        let parcel = ProtoParcel::dsc_res(neighbours, self_node);

                        write_parcel(&mut stream, &parcel);
                    } else {
                        error!("Body-header type mismatch!");
                        return;
                    }
                }

                Type::SeqReq => {
                    info!("Received SeqReq with id {} from node {}", parcel.id, parcel.sender_id);
                    let state = state_ref.write().unwrap(); // acquire write lock
                    let seq = state.sequence;
                    let parcel = ProtoParcel::seq_res(seq);
                    drop(state);
                    write_parcel(&mut stream, &parcel);
                }
                Type::Ping => {
                    info!("Received Ping with id {} from node {}", parcel.id, parcel.sender_id);
                    let state = state_ref.write().unwrap(); // acquire write lock
                    let parcel = ProtoParcel::pong();
                    drop(state);
                    write_parcel(&mut stream, &parcel);
                }
                Type::ProtoError => {
                    error!("Proto Error")
                }
                Type::StateChange => {
                    if let Body::StateChange { mode } = parcel.body {
                        info!("Received StateChange with id {} from node {}", parcel.id, parcel.sender_id);
                        let mut state = state_ref.write().unwrap();
                        state.neighbours.entry(parcel.sender_id).and_modify(|node| {
                            node.mode = mode
                        });
                        drop(state);
                        let ack = ProtoParcel::ack(parcel.id);
                        write_parcel(&mut stream, &ack);
                    }
                }
                Type::AddNode => {
                    if let Body::AddNode { nodes } = parcel.body {
                        info!("Received AddNode with id {} from node {}", parcel.id, parcel.sender_id);
                        let mut state = state_ref.write().unwrap();
                        for node in nodes {
                            state.add_neighbour(node);
                        }
                        drop(state);
                        let ack = ProtoParcel::ack(parcel.id);
                        write_parcel(&mut stream, &ack);
                    }
                }
                Type::ResourceRequest => {
                    if let Body::ResourceRequest { message_hash, sequence, timestamp } = parcel.body {
                        info!("Processing Resource Request with id {} from node {}", parcel.id, parcel.sender_id);

                        let lock = f_access.lock().unwrap();

                        let resource_pledge: ResourceRequest = ResourceRequest {
                            owner: parcel.sender_id,
                            message_hash,
                            timestamp,
                            sequence,
                        };

                        let mut pledge_queue = pledge_queue.lock().unwrap();
                        pledge_queue.push(resource_pledge);
                        drop(pledge_queue);

                        let ack = ProtoParcel::ack(parcel.id);
                        write_parcel(&mut stream, &ack);
                        drop(lock);


                    }
                }
                Type::ResourceRelease => {
                    if let Body::ResourceRelease { timestamp, message_hash, sequence, message } = parcel.body {
                        info!("Processing Resource Release with id {} from node {}", parcel.id, parcel.sender_id);

                        let state = state_ref.read().unwrap();

                        if state.current_lock == message_hash {
                            let _resource_release: ResourceRelease = ResourceRelease {
                                owner: parcel.sender_id,
                                message_hash,
                                timestamp,
                                message: MessageWrapper {
                                    message,
                                    sequence,
                                    receiver_mask: 0,
                                },
                                local: false,
                                sequence,
                            };


                            let parcel = ProtoParcel::ack(parcel.id);
                            write_parcel(&mut stream, &parcel);
                        } else {
                            warn!("Neighbour attempted to release resource without lock");
                            let parcel = ProtoParcel::proto_error();
                            write_parcel(&mut stream, &parcel);
                        }
                    }
                }
                Type::ExtAddrReq => {
                    info!("Got ExtAddrReq with id {} from node {}", parcel.id, parcel.sender_id);
                    let addr = stream.peer_addr().unwrap();
                    let res = ProtoParcel::ext_addr_res(addr);
                    write_parcel(&mut stream, &res);
                }
                _ => {
                    error!("Unexpected message type!, {}", parcel.parcel_type);
                    return;
                }
            }
        });
    }
}
