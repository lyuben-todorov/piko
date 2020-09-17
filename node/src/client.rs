use std::sync::{RwLock, Arc, Mutex};

use std::collections::{VecDeque, HashMap, BinaryHeap};

use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use byteorder::ReadBytesExt;
use std::io::{Read, Write};

use std::sync::mpsc::{Sender};

use sha2::{Sha256, Digest};

use chrono::{Utc};
use crate::proto::{ResourceRequest, ResourceRelease, MessageWrapper};
use crate::req::publish::publish;
use crate::state::State;

use log::{info};

struct Client<'a> {
    identity: u64,
    message_queue: VecDeque<&'a Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
enum ReqType {
    Poll,
    LongPoll,
    Subscribe,
    Unsubscribe,
    Publish,
}

#[derive(Serialize, Deserialize)]
enum ReqBody {
    Poll {
        client_id: u64
    },
    LongPoll {
        client_id: u64
    },
    Subscribe {
        client_id: u64
    },
    Unsubscribe {
        client_id: u64
    },
    Publish {
        client_id: u64,
        message: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct ClientReq {
    req_type: ReqType,
    req_body: ReqBody,
}

fn read_req(stream: &mut TcpStream) -> ClientReq {
    let count = stream.read_u8().unwrap();

    // debug!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];
    stream.read_exact(&mut buf).unwrap();

    let client_req: ClientReq = serde_cbor::from_slice(buf.as_slice()).unwrap();
    client_req
}

fn write_bytes(stream: &mut TcpStream, buf: &[u8]) {
    stream.write_all(buf).unwrap();
}

pub fn client_listener(listener: TcpListener, state: Arc<RwLock<State>>,  pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>, f_access: Arc<Mutex<bool>>) {
    let client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>> = Arc::new(RwLock::new(HashMap::<u64, RwLock<Client>>::new()));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let client_list = client_list.clone();
        let state_ref = state.clone();
        let pledge_queue = Arc::clone(&pledge_queue);

        rayon::spawn(move || {
            let req = read_req(&mut stream);

            match req.req_type {
                ReqType::Subscribe => {
                    if let ReqBody::Subscribe { client_id } = req.req_body {
                        let client: Client = Client {
                            identity: client_id,
                            message_queue: VecDeque::<&Vec<u8>>::new(),
                        };

                        client_list.write().unwrap().insert(client_id, RwLock::from(client));
                    }
                }
                ReqType::Unsubscribe => {
                    if let ReqBody::Unsubscribe { client_id } = req.req_body {
                        client_list.write().unwrap().remove(&client_id);
                    }
                }
                ReqType::Poll => {
                    if let ReqBody::Poll { client_id } = req.req_body {
                        let client_list = client_list.read().unwrap();
                        let mut message_dequeue = client_list.get(&client_id).unwrap().write().unwrap();
                        let message = message_dequeue.message_queue.pop_front();
                        match message {
                            None => {
                                // write empty buffer
                                write_bytes(&mut stream, &[])
                            }
                            Some(message) => {
                                write_bytes(&mut stream, message.as_slice())
                            }
                        }
                    }
                }
                ReqType::LongPoll => {}
                ReqType::Publish => {
                    if let ReqBody::Publish { client_id, message } = req.req_body {
                        info!("Publishing message from client {} with size {}", client_id, message.len());

                        let state_ref = state_ref.write().unwrap();

                        let message_hash: [u8; 32] = Sha256::digest(message.as_slice()).into();
                        let timestamp = Utc::now();

                        let req: ResourceRequest = ResourceRequest {
                            owner: *crate::proto::SENDER.lock().unwrap(),
                            message_hash,
                            timestamp,
                            sequence: 0,
                        };

                        let mut pledge_queue = pledge_queue.lock().unwrap();
                        pledge_queue.push(req);
                        drop(pledge_queue);

                        publish(&state_ref.get_neighbour_addrs(), req);
                        let release: ResourceRelease = ResourceRelease {
                            owner: *crate::proto::SENDER.lock().unwrap(),
                            message_hash,
                            timestamp,
                            message: MessageWrapper {
                                message,
                                sequence: 0,
                                receiver_mask: 0,
                            },
                            local: false,
                            sequence: 0,
                        };
                    }
                }
            }
        })
    }
}
