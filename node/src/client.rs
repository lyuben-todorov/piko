use std::sync::{RwLock, Arc, Mutex};

use std::collections::{VecDeque, HashMap, BinaryHeap};

use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::error::Error;

use sha2::{Sha256, Digest};

use chrono::{Utc};
use crate::proto::{ResourceRequest, ResourceRelease, MessageWrapper};
use crate::req::publish::publish;
use crate::state::State;

use log::{info, error};

struct Client<'a> {
    identity: u64,
    message_queue: VecDeque<&'a Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub enum ClientRes {
    Success {
        message: String,
        bytes: Vec<u8>
    },
    Error {
        message: String
    },
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

impl ClientReq {
    pub fn sub(client_id: u64) -> ClientReq {
        ClientReq {
            req_type: ReqType::Subscribe,
            req_body: ReqBody::Subscribe { client_id },
        }
    }
}

///
/// Read Request from client
///
fn read_req(stream: &mut TcpStream) -> Result<ClientReq, Box<dyn Error>> {
    let count = stream.read_u8()?;

    // debug!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];

    stream.read_exact(&mut buf)?;

    let client_req: ClientReq = serde_cbor::from_slice(buf.as_slice())?;

    Ok(client_req)
}

///
/// Write Response to client
///
fn write_res(stream: &mut TcpStream, res: ClientRes) {

    let buf = serde_cbor::to_vec(&res).unwrap();

    stream.write_u8(buf.len() as u8);
    stream.write_all(buf.as_slice()).unwrap();
}

pub fn client_listener(listener: TcpListener, state: Arc<RwLock<State>>, pledge_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>, _f_access: Arc<Mutex<bool>>) {
    let client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>> = Arc::new(RwLock::new(HashMap::<u64, RwLock<Client>>::new()));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let client_list = client_list.clone();
        let state_ref = state.clone();
        let pledge_queue = Arc::clone(&pledge_queue);

        rayon::spawn(move || {
            let req = match read_req(&mut stream) {
                Ok(req) => req,
                Err(e) => {
                    return;
                }
            };

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
                                let res= ClientRes::Success {
                                    message: "".to_string(),
                                    bytes: vec![]
                                };
                                write_res(&mut stream, res);
                            }
                            Some(message) => {
                                let res = ClientRes::Success {
                                    message: "".to_string(),
                                    bytes: message.to_vec()
                                };
                                write_res(&mut stream, res)
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
                        let _release: ResourceRelease = ResourceRelease {
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
