use std::{fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::{Node, Mode};

use std::fmt::Display;

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{random};

use std::sync::Mutex;
use lazy_static::lazy_static;


static PROTO_VERSION: &str = "1.0";

lazy_static! {
    static ref SENDER: Mutex<u16> = Mutex::new(0);
}

pub fn set_sender_id(id: u16) {
    let mut _id = SENDER.lock().unwrap();
    *_id = id;
}

// Enumeration over the types of protocol messages
#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    ProtoError = 0,
    DscReq = 1,
    DscRes = 2,
    SeqReq = 3,
    SeqRes = 4,
    StateChange = 5,
    Ping = 6,
    Pong = 7,
    SequenceLock = 8,
    Message = 9,
    Ack = 10,

}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::DscReq => write!(f, "{}", "DscReq"),
            Type::DscRes => write!(f, "{}", "DscRes"),
            Type::SeqReq => write!(f, "{}", "SeqReq"),
            Type::SeqRes => write!(f, "{}", "SeqRes"),
            Type::ProtoError => write!(f, "{}", "Err"),
            Type::StateChange => write!(f, "{}", "StateChange"),
            Type::Ping => write!(f, "{}", "Ping"),
            Type::Pong => write!(f, "{}", "Pong"),
            Type::SequenceLock => write!(f, "{}", "SequenceLock"),
            Type::Message => write!(f, "{}", "Message"),
            Type::Ack => write!(f, "{}", "Ack"),
        }
    }
}

// Enumeration over the types of protocol errors.
#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProtoError {
    BadRes = 1,
    BadReq = 2,

}

#[derive(Serialize, Deserialize)]
pub enum Body {
    Empty,

    DscReq {
        identity: Node
    },

    DscRes {
        neighbours: Vec<Node>,
    },

    SeqRes {
        seq_number: u8
    },

    StateChange {
        mode: Mode
    },

    Message {
        bytes: Vec<u8>,
        sender: u16,
    },

    Ack {
        message_id: u64
    },
}


#[derive(Serialize, Deserialize)]
pub struct ProtoParcel {
    // id of message
    pub id: u64,
    // id of sender
    pub sender_id: u16,
    // whether or not packet is a response
    pub is_response: bool,
    // type of packet
    pub parcel_type: Type,
    // size of application-specific data in bytes
    pub size: u16,
    // message body
    pub body: Body,
}

impl ProtoParcel {
    pub fn dsc_req(self_node_information: Node) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::DscReq,
            size: 0,
            body: Body::DscReq { identity: self_node_information },
        }
    }

    pub fn dsc_res(neighbours_information: Vec<Node>) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::DscRes,
            size: 0,
            body: Body::DscRes { neighbours: neighbours_information },
        }
    }

    pub fn seq_req() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::SeqReq,
            size: 0,
            body: Body::Empty,
        }
    }

    pub fn seq_res(seq_number: u8) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::SeqRes,
            size: 0,
            body: Body::SeqRes { seq_number },
        }
    }

    pub fn state_change(mode: Mode) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::StateChange,
            size: 0,
            body: Body::StateChange { mode },
        }
    }

    pub fn ping() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::Ping,
            size: 0,
            body: Body::Empty,
        }
    }

    pub fn pong() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Pong,
            size: 0,
            body: Body::Empty,
        }
    }

    pub fn ack(message_id: u64) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Ack,
            size: 0,
            body: Body::Ack { message_id },
        }
    }
}

pub fn generate_id() -> u64 {
    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let random: u64 = random();
    id ^ random
}