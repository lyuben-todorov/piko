use std::net::{TcpListener, TcpStream};

use crate::state::{State, Node};


use std::sync::{Arc, RwLock, Mutex};
use std::io::{Read, Write};
use crate::proto::{ProtoParcel, Type, Body, ResourceRequest, ResourceRelease};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use crate::internal::TaskSignal;
use crate::req::add_node::add_node;

use log::{error, info, warn, debug};
use std::collections::{BinaryHeap};
use std::error::Error;
use std::sync::mpsc::Sender;
use chrono::{DateTime, Utc};
use crate::semaphore::OrdSemaphore;


pub fn read_parcel(stream: &mut TcpStream) -> Result<ProtoParcel, Box<dyn Error>> {
    let size: u64 = stream.read_u64::<LittleEndian>()?;

    // debug!("Expecting {} bytes", size);
    let mut buf = vec![0u8; size as usize];
    stream.read_exact(&mut buf)?;

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice())?;
    Ok(proto_parcel)
}

pub fn write_parcel(stream: &mut TcpStream, parcel: &ProtoParcel) {
    let parcel = serde_cbor::to_vec(&parcel).unwrap();
    let buf = parcel.as_slice();
    let count: u64 = buf.len() as u64;

    // debug!("Writing {} bytes", count);
    stream.write_u64::<LittleEndian>(count).unwrap();
    stream.write_all(buf).unwrap();
}

pub fn is_acked(response: ProtoParcel, ack_id: u64) -> TaskSignal {
    match response.parcel_type {
        Type::Ack => {
            if let Body::Ack { message_id } = response.body {
                if message_id == ack_id {
                    info!("Acked {}", message_id);
                    TaskSignal::Success
                } else {
                    TaskSignal::Fail
                }
            } else {
                error!("Body-header type mismatch!");
                TaskSignal::Fail
            }
        }
        Type::ProtoError => {
            TaskSignal::Fail
        }
        _ => {
            error!("Expected acknowledge, got {}", response.parcel_type);
            TaskSignal::Fail
        }
    }
}

pub fn listener_thread(socket: TcpListener, state: Arc<RwLock<State>>, resource_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
                       semaphore: Arc<OrdSemaphore<DateTime<Utc>>>, wrk: Sender<ResourceRelease>) {
    info!("Started Listener thread!");

    for stream in socket.incoming() {
        let mut stream = stream.unwrap();

        let state_ref = Arc::clone(&state);
        let pledge_queue = Arc::clone(&resource_queue);
        let wrk = wrk.clone();
        let semaphore = semaphore.clone();

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

                        let mut neighbours = vec![];
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
                    // debug!("Received Ping with id {} from node {}", parcel.id, parcel.sender_id);
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
                    if let Body::ResourceRequest { resource_request } = parcel.body {
                        info!("Processing Resource Request with id {} from node {}", parcel.id, parcel.sender_id);

                        semaphore.wait_until(&resource_request.timestamp);

                        debug!("4 lock");
                        let mut pledge_queue = pledge_queue.lock().unwrap();
                        pledge_queue.push(resource_request);
                        drop(pledge_queue);
                        debug!("4 rel");
                        let ack = ProtoParcel::ack(parcel.id);
                        write_parcel(&mut stream, &ack);
                    }
                }
                Type::ResourceRelease => {
                    if let Body::ResourceRelease { resource_release } = parcel.body {
                        info!("Processing Resource Release with id {} from node {}", parcel.id, parcel.sender_id);

                        debug!("3 lock");
                        let pledge_queue = pledge_queue.lock().unwrap();
                        if pledge_queue.peek().unwrap().message_hash == resource_release.message_hash {
                            wrk.send(resource_release).unwrap();
                            let parcel = ProtoParcel::ack(parcel.id);
                            write_parcel(&mut stream, &parcel);
                            debug!("3 rel");
                        } else {
                            warn!("Neighbour attempted to release resource without lock");
                            let parcel = ProtoParcel::proto_error();
                            write_parcel(&mut stream, &parcel);
                            debug!("3 rel");
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
