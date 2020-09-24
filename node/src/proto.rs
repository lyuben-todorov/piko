use std::{fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::{Node, Mode};

use std::fmt::Display;

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{random};

use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use enum_dispatch::enum_dispatch;


static _PROTO_VERSION: &str = "1.0";

lazy_static! {
    pub static ref SENDER: Mutex<u16> = Mutex::new(0);
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

    ResourceRequest = 8,
    ResourceRelease = 9,

    Ack = 10,
    Publish = 11,

    AddNode = 12,
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
            Type::Ack => write!(f, "{}", "Ack"),
            Type::AddNode => write!(f, "{}", "AddNode"),
            Type::ResourceRelease => write!(f, "{}", "ResourceRelease"),
            Type::ResourceRequest => write!(f, "{}", "ResourceRequest"),

            Type::Publish => write!(f, "{}", "Publish"),
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
        self_id: Node,
        neighbours: Vec<Node>,
    },

    SeqRes {
        seq_number: u8
    },

    AddNode {
        nodes: Vec<Node>
    },

    StateChange {
        mode: Mode
    },

    Publish {
        message: Vec<u8>,
    },

    ResourceRequest {
        timestamp: DateTime<Utc>,
        message_hash: [u8; 32],
        sequence: u16,
    },

    ResourceRelease {
        timestamp: DateTime<Utc>,
        message_hash: [u8; 32],
        sequence: u16,
        message: Vec<u8>,
    },

    Ack {
        message_id: u64
    },
}

#[derive(Clone)]
pub struct MessageWrapper {
    pub message: Vec<u8>,
    pub sequence: u16,
    pub receiver_mask: u32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ResourceRequest {
    pub owner: u16,
    pub message_hash: [u8; 32],
    pub timestamp: DateTime<Utc>,
    pub sequence: u16,
}

impl Ord for ResourceRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

impl PartialOrd for ResourceRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ResourceRelease {
    pub owner: u16,
    pub message_hash: [u8; 32],
    pub timestamp: DateTime<Utc>,
    pub message: MessageWrapper,
    pub local: bool,
    pub sequence: u16,
}

#[enum_dispatch]
pub enum Pledge {
    ResourceRequest(ResourceRequest),
    ResourceRelease(ResourceRelease),
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
            body: Body::DscReq { identity: self_node_information },
        }
    }

    pub fn dsc_res(neighbours_information: Vec<Node>, self_id:Node) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::DscRes,
            body: Body::DscRes { neighbours: neighbours_information, self_id },
        }
    }

    pub fn seq_req() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::SeqReq,
            body: Body::Empty,
        }
    }

    pub fn seq_res(seq_number: u8) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::SeqRes,
            body: Body::SeqRes { seq_number },
        }
    }

    pub fn add_node(nodes: Vec<Node>) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::AddNode,
            body: Body::AddNode { nodes },
        }
    }

    pub fn state_change(mode: Mode) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::StateChange,
            body: Body::StateChange { mode },
        }
    }

    pub fn ping() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::Ping,
            body: Body::Empty,
        }
    }

    pub fn pong() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Pong,
            body: Body::Empty,
        }
    }

    pub fn ack(message_id: u64) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Ack,
            body: Body::Ack { message_id },
        }
    }
    pub fn resource_request(request: ResourceRequest) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRequest,
            body: Body::ResourceRequest {
                timestamp: request.timestamp,
                message_hash: request.message_hash,
                sequence: request.sequence,
            },
        }
    }
    pub fn resource_release(message_hash: [u8; 32], timestamp: DateTime<Utc>, sequence: u16, message: Vec<u8>) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRelease,
            body: Body::ResourceRelease {
                timestamp,
                message_hash,
                sequence,
                message,
            },
        }
    }
    pub fn proto_error() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::ProtoError,
            body: Body::Empty,
        }
    }
}

pub fn generate_id() -> u64 {
    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let random: u64 = random();
    id ^ random
}