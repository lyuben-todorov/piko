use crate::state::{State, Node};

use std::sync::{Arc, RwLock, Mutex};
use std::io::{Read, Write};
use crate::proto::{ProtoParcel, Type, Body, ResourceRequest, ResourceRelease};
use crate::internal::TaskSignal;
use crate::req::add_node::add_node;

use log::{error, info, debug};
use std::collections::{BinaryHeap};
use std::error::Error;
use crossbeam_channel::{Sender};
use chrono::{DateTime, Utc};
use crate::semaphore::OrdSemaphore;
use tokio::net::{TcpStream, TcpListener};
use tokio::stream::StreamExt;
use tokio_byteorder::{BigEndian, AsyncReadBytesExt, AsyncWriteBytesExt};
use tokio::prelude::*;

pub async fn read_parcel(stream: &mut TcpStream) -> Result<ProtoParcel, Box<dyn Error>> {
    let size: u64 = stream.read_u64().await?;

    // debug!("Expecting {} bytes", size);
    let mut buf = vec![0u8; size as usize];
    stream.read_exact(&mut buf).await?;

    let proto_parcel: ProtoParcel = serde_cbor::from_slice(buf.as_slice())?;
    Ok(proto_parcel)
}

pub async fn write_parcel(stream: &mut TcpStream, parcel: &ProtoParcel) {
    let parcel = serde_cbor::to_vec(&parcel).unwrap();
    let buf = parcel.as_slice();
    let count: u64 = buf.len() as u64;

    // debug!("Writing {} bytes", count);
    stream.write_u64(count).await;
    stream.write_all(buf).await;
}

pub fn is_acked(response: ProtoParcel, ack_id: u64) -> TaskSignal {
    match response.parcel_type {
        Type::Ack => {
            if let Body::Ack { message_id } = response.body {
                if message_id == ack_id {
                    debug!("Acked {}", message_id);
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

pub async fn listener_thread(mut listener: TcpListener, state: Arc<RwLock<State>>, resource_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
                             semaphore: Arc<OrdSemaphore<DateTime<Utc>>>, wrk: Sender<ResourceRelease>) {
    info!("Started Listener thread!");


    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // Clone shared handles.
        let state_ref = state.clone();
        let pledge_queue = resource_queue.clone();
        let wrk = wrk.clone();
        let semaphore = semaphore.clone();

        println!("Accepted");
        tokio::spawn(async move {
            process_stream(socket, state_ref, pledge_queue, wrk, semaphore);
        });
    }
}

async fn process_stream(mut stream: TcpStream, state_ref: Arc<RwLock<State>>,
                        pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>,
                        wrk: Sender<ResourceRelease>,
                        semaphore: Arc<OrdSemaphore<DateTime<Utc>>>) {
    let parcel = match read_parcel(&mut stream).await {
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

                write_parcel(&mut stream, &parcel).await;
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
            write_parcel(&mut stream, &parcel).await;
        }
        Type::Ping => {
            // debug!("Received Ping with id {} from node {}", parcel.id, parcel.sender_id);
            let state = state_ref.write().unwrap(); // acquire write lock
            let parcel = ProtoParcel::pong();
            drop(state);
            write_parcel(&mut stream, &parcel).await;
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
                write_parcel(&mut stream, &ack).await;
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
                write_parcel(&mut stream, &ack).await;
            }
        }
        Type::ResourceRequest => {
            if let Body::ResourceRequest { resource_request } = parcel.body {
                info!("Processing Resource Request with id {} from node {}", parcel.id, parcel.sender_id);

                semaphore.wait_until(&resource_request.timestamp);

                let mut pledge_queue = pledge_queue.lock().unwrap();

                pledge_queue.push(resource_request);
                drop(pledge_queue);
                let ack = ProtoParcel::ack(parcel.id);
                write_parcel(&mut stream, &ack).await;
            }
        }
        Type::ResourceRelease => {
            if let Body::ResourceRelease { resource_release } = parcel.body {
                info!("Processing Resource Release with hash {} from node {}", resource_release.shorthand, parcel.sender_id);

                wrk.send(resource_release).unwrap();
                let parcel = ProtoParcel::ack(parcel.id);
                write_parcel(&mut stream, &parcel).await;
            }
        }
        Type::ExtAddrReq => {
            info!("Got ExtAddrReq with id {} from node {}", parcel.id, parcel.sender_id);
            let addr = stream.peer_addr().unwrap();
            let res = ProtoParcel::ext_addr_res(addr);
            write_parcel(&mut stream, &res).await;
        }
        _ => {
            error!("Unexpected message type!, {}", parcel.parcel_type);
            return;
        }
    }
}