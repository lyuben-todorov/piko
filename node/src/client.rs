use std::sync::{RwLock, Arc, Mutex};

use std::collections::{VecDeque, HashMap, BinaryHeap};

use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use std::error::Error;

use crate::proto::{ResourceRequest, ResourceRelease};
use crate::req::publish::pub_req;
use crate::state::State;

use log::{info, error, debug};

use crate::internal::TaskSignal;
use chrono::{Utc, DateTime};
use crate::semaphore::OrdSemaphore;



pub struct Client<'a> {
    identity: u64,
    message_queue: VecDeque<&'a Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
pub enum ClientRes {
    Success {
        message: String,
        bytes: Vec<u8>,
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
pub enum ClientReq {
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

impl ClientReq {
    pub fn sub(client_id: u64) -> ClientReq {
        ClientReq::Subscribe {
            client_id
        }
    }
    pub fn unsub(client_id: u64) -> ClientReq {
        ClientReq::Unsubscribe {
            client_id
        }
    }
    pub fn publ(client_id: u64, message: Vec<u8>) -> ClientReq {
        ClientReq::Publish {
            client_id,
            message,
        }
    }
    pub fn poll(client_id: u64) -> ClientReq {
        ClientReq::Poll {
            client_id
        }
    }
}

// Read Request from client
fn read_req(stream: &mut TcpStream) -> Result<ClientReq, Box<dyn Error>> {
    let count = stream.read_u8()?;

    // debug!("Expecting {} bytes", count);

    let mut buf = vec![0u8; count as usize];

    stream.read_exact(&mut buf)?;

    let client_req: ClientReq = serde_cbor::from_slice(buf.as_slice())?;

    Ok(client_req)
}

// Write Response to client
fn write_res(stream: &mut TcpStream, res: ClientRes) {
    let buf = serde_cbor::to_vec(&res).unwrap();

    // debug!("Writing {} bytes to client", buf.len());
    stream.write_u8(buf.len() as u8).unwrap();
    stream.write_all(buf.as_slice()).unwrap();
}

fn ok(stream: &mut TcpStream) {
    write_res(stream, ClientRes::Success { message: "Ok".to_string(), bytes: vec![] });
}

fn ok_with_message(stream: &mut TcpStream, message: &str) {
    write_res(stream, ClientRes::Success { message: message.to_string(), bytes: vec![] });
}

fn err(stream: &mut TcpStream, message: &str) {
    write_res(stream, ClientRes::Error { message: message.to_string() });
}

pub fn client_listener(listener: TcpListener, state: Arc<RwLock<State>>, // Node state & listener
                       resource_queue: Arc<Mutex<BinaryHeap<ResourceRequest>>>, // Resource queue
                       semaphore: Arc<OrdSemaphore<DateTime<Utc>>>, // Total order slemaphore
                       pending_messages: Arc<Mutex<HashMap<u16, (ResourceRelease, bool)>>>) {
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let client_list: Arc<RwLock<HashMap<u64, RwLock<Client>>>> = Arc::new(RwLock::new(HashMap::<u64, RwLock<Client>>::new()));
        let state_ref = state.clone();
        let pledge_queue = Arc::clone(&resource_queue);
        let pending_messages = pending_messages.clone();
        let semaphore = semaphore.clone();

        rayon::spawn(move || {
            // debug!("Received message from client!");
            let req = match read_req(&mut stream) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed reading message from client! {}", e);
                    return;
                }
            };

            match req {
                ClientReq::Subscribe { client_id } => {
                    debug!("Sub request from client {}", client_id);
                    let client: Client = Client {
                        identity: client_id,
                        message_queue: VecDeque::<&Vec<u8>>::new(),
                    };

                    let mut client_list =client_list.write().unwrap();
                    let write = client_list.insert(client_id, RwLock::from(client));
                    match write {
                        None => ok(&mut stream),
                        Some(_) => ok_with_message(&mut stream, "Client exists, flushed queue")
                    }
                }
                ClientReq::Unsubscribe { client_id } => {
                    debug!("Unsub request from client {}", client_id);

                    let v = client_list.write().unwrap().remove(&client_id);
                    match v {
                        None => err(&mut stream, "Client wasn't previously subscribed"),
                        Some(_) => ok(&mut stream)
                    }
                }
                ClientReq::Poll { client_id } => {
                    let client_list = client_list.read().unwrap();
                    let client = match client_list.get(&client_id) {
                        None => {
                            err(&mut stream, "client not subscribed.");
                            return;
                        }
                        Some(client) => { client }
                    };
                    let mut client = client.write().unwrap();

                    let message = client.message_queue.pop_front();

                    match message {
                        None => {
                            // write empty buffer
                            let res = ClientRes::Success {
                                message: "Queue empty".to_string(),
                                bytes: vec![],
                            };
                            write_res(&mut stream, res);
                        }
                        Some(message) => {
                            let res = ClientRes::Success {
                                message: "Message:".to_string(),
                                bytes: message.to_vec(),
                            };
                            write_res(&mut stream, res)
                        }
                    }
                }
                ClientReq::LongPoll { client_id: _ } => {}
                ClientReq::Publish { client_id, message } => {
                    info!("Publishing message from client {} with size {}", client_id, message.len());
                    let (req, rel) = ResourceRequest::generate(message);
                    let key = rel.shorthand;

                    let client = semaphore.create_task(req.timestamp);

                    // Place REQUEST on local queue
                    debug!("2 lock");
                    let mut pledge_queue = pledge_queue.lock().unwrap();
                    pledge_queue.push(req);
                    drop(pledge_queue);
                    debug!("2 rel");

                    // Place eventual RELEASE on KV store
                    let mut messages = pending_messages.lock().unwrap();
                    messages.insert(key, (rel, false));
                    drop(messages);

                    // debug!("Sleeping to simulate concurrent request!");
                    // std::thread::sleep(Duration::from_secs(5));

                    // Publish REQUEST
                    let state_ref = state_ref.read().unwrap();
                    let neighbours = &state_ref.get_neighbour_addrs();
                    drop(state_ref);

                    let result = pub_req(neighbours, req);

                    match result {
                        TaskSignal::Success => {
                            client.consume();
                            let mut messages = pending_messages.lock().unwrap();
                            messages.entry(key).and_modify(|x|x.1 = true);
                            info!("Resource REQUEST acknowledged!");
                        }
                        _ => {
                            error!("Resource REQUEST failed!");
                        }
                    }
                    // ack client
                    ok(&mut stream);
                }
            }
        })
    }
}
