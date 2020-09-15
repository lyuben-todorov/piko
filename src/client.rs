use std::sync::{RwLock, Arc};

use std::collections::{VecDeque, HashMap};




use std::net::{TcpListener, TcpStream};
use serde::{Serialize, Deserialize};
use byteorder::ReadBytesExt;
use std::io::{Read, Write};

struct Client {
    identity: u64,
    message_queue: VecDeque<Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
enum ReqType {
    Poll,
    LongPoll,
    Subscribe,
    Unsubscribe,
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


///
///  Tasked with dispatching messages to clients.
///
// fn client_dispatch(state: Arc<RwLock<State>>, client_list: Vec<Client>, message_receiver: Receiver<MessageWrapper>) {
//     for message in message_receiver.iter() {
//         client_list.into_par_iter().for_each_with(message, |m, mut client: Client| {
//             client.message_queue.push_back(m.message)
//         });
//     }
// }

pub fn client_listener(listener: TcpListener) {
    let client_list: Arc<RwLock<HashMap<u64, Client>>> = Arc::new(RwLock::new(HashMap::new()));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let client_list_ref = client_list.clone();
        rayon::spawn(move || {
            let req = read_req(&mut stream);
            match req.req_type {
                ReqType::Subscribe => {
                    if let ReqBody::Subscribe { client_id } = req.req_body {
                        let client: Client = Client {
                            identity: client_id,
                            message_queue: VecDeque::<Vec<u8>>::new(),
                        };

                        let mut client_list = client_list_ref.write().unwrap();
                        client_list.insert(client_id, client);
                    }
                }
                ReqType::Unsubscribe => {
                    if let ReqBody::Unsubscribe { client_id } = req.req_body {
                        let mut client_list = client_list_ref.write().unwrap();
                        client_list.remove(&client_id);
                    }
                }
                ReqType::Poll => {
                    if let ReqBody::Poll { client_id: _ } = req.req_body {
                        // let mut client_list = client_list_ref.read().unwrap();
                        //
                        // let message_dequeue = client_list.get(&client_id).unwrap().message_queue;
                        // let message = message_dequeue.pop_front();
                        // match message {
                        //     None => {
                        //         // write empty buffer
                        //         write_bytes(&mut stream, &[])
                        //     }
                        //     Some(message) => {
                        //         write_bytes(&mut stream, message.as_slice())
                        //     }
                        // }
                    }
                }
                ReqType::LongPoll => {}
            }
        })
    }
}
