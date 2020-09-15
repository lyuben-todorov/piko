use std::sync::{RwLock, Arc};

use std::collections::{VecDeque, HashMap};

use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use byteorder::ReadBytesExt;
use std::io::{Read, Write};


use std::sync::mpsc::{Sender};

use crate::wrk::{Pledge, ResourceRequest};
use sha2::{Sha256, Digest};

use chrono::{Utc};

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

pub fn client_listener(listener: TcpListener, sender: Sender<Pledge>) {
    let client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>> = Arc::new(RwLock::new(HashMap::<u64, RwLock<Client>>::new()));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let client_list = client_list.clone();
        let sender = sender.clone();
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
                    if let ReqBody::Publish { client_id: _, message } = req.req_body {
                        let hash: [u8; 32] = Sha256::digest(message.as_slice()).into();
                        let pledge: ResourceRequest = ResourceRequest {
                            owner: *crate::proto::SENDER.lock().unwrap(),
                            message_hash: hash,
                            timestamp: Utc::now(),
                            sequence: 0
                        };

                        sender.send(Pledge::ResourceRequest(pledge));
                    }
                }
            }
        })
    }
}
