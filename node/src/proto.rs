use std::{fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::{Node, Mode};

use std::fmt::Display;

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{random};

use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::{DateTime, Utc, Timelike};
use std::cmp::Ordering;

use std::net::SocketAddr;
use sha2::{Sha256, Digest};
use std::convert::TryInto;
use sha2::digest::DynDigest;


lazy_static! {
    // WIP :/
    pub static ref PROTO_VERSION: String = "1.1".to_string();
    pub static ref SENDER: Mutex<u64> = Mutex::new(0);
}
const PRIME_ONE: u64 = 2999085892127319403;
const PRIME_TWO: u64 = 13962674565864582377;
const PRIME_THREE: u64 = 13714677094544069263;

pub fn get_proto_version() -> String {
    return PROTO_VERSION.clone();
}

pub fn set_sender_id(id: u64) {
    let mut _id = SENDER.lock().unwrap();
    *_id = id;
}

fn calculate_hash(message: &Vec<u8>, timestamp: &DateTime<Utc>) -> ([u8; 32], u64) {
    let mut hasher = Sha256::new();

    DynDigest::update(&mut hasher, &message.as_slice());
    DynDigest::update(&mut hasher, &timestamp.nanosecond().to_be_bytes());

    let message_hash: [u8; 32] = hasher.finalize().into();

    let shorthand = shorten_hash(&message_hash);

    return (message_hash, shorthand);
}

pub fn shorten_hash(message_hash: &[u8; 32]) -> u64 {
    message_hash.chunks(8).into_iter()
        .map(|x: &[u8]| u64::from_be_bytes(x.try_into().unwrap()))
        .fold(0, |x: u64, y: u64|
            // compute (x * p1 + y * p2) % p3 with overflows
            u64::overflowing_rem(
                u64::overflowing_add(
                    u64::overflowing_mul(x, PRIME_ONE).0,
                    u64::overflowing_mul(y, PRIME_TWO).0).0,
                PRIME_THREE).0,
        )
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

    AddNode = 11,

    ExtAddrReq = 12,
    ExtAddrRes = 13,
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

            Type::ExtAddrRes => write!(f, "{}", "ExtAddrRes"),
            Type::ExtAddrReq => write!(f, "{}", "ExtAddrReq"),
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
        resource_request: ResourceRequest
    },

    ResourceRelease {
        resource_release: ResourceRelease
    },

    ExtAddrRes {
        addr: SocketAddr
    },

    Ack {
        message_id: u64
    },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageWrapper {
    pub message: Vec<u8>,
    pub sequence: u16,
    pub receiver_mask: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub owner: u64,
    pub message_hash: [u8; 32],
    pub shorthand: u64,
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

#[derive(Serialize, Deserialize)]
pub struct ResourceRelease {
    pub owner: u64,
    pub message_hash: [u8; 32],
    pub shorthand: u64,
    pub timestamp: DateTime<Utc>,
    pub message: MessageWrapper,
    pub local: bool,
    pub sequence: u16,
}

impl ResourceRequest {
    pub fn generate(message: Vec<u8>) -> (ResourceRequest, ResourceRelease) {
        let timestamp = Utc::now();

        let (message_hash, shorthand) = calculate_hash(&message, &timestamp);

        let id = *crate::proto::SENDER.lock().unwrap();
        (
            ResourceRequest {
                owner: id,
                message_hash,
                shorthand,
                timestamp,
                sequence: 0,
            },
            ResourceRelease {
                owner: id,
                message_hash,
                shorthand,
                timestamp,
                message: MessageWrapper {
                    message,
                    sequence: 0,
                    receiver_mask: 0,
                },
                local: false,
                sequence: 0,
            }
        )
    }
}

pub enum Pledge {
    Ack(u16),
    ResourceRelease(ResourceRelease),
}


#[derive(Serialize, Deserialize)]
pub struct ProtoParcel {
    // id of message
    pub id: u64,
    pub proto_version: String,
    // id of sender
    pub sender_id: u64,
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
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::DscReq,
            body: Body::DscReq { identity: self_node_information },
        }
    }

    pub fn dsc_res(neighbours_information: Vec<Node>, self_id: Node) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::DscRes,
            body: Body::DscRes { neighbours: neighbours_information, self_id },
        }
    }

    pub fn seq_req() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::SeqReq,
            body: Body::Empty,
        }
    }

    pub fn seq_res(seq_number: u8) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::SeqRes,
            body: Body::SeqRes { seq_number },
        }
    }

    pub fn add_node(nodes: Vec<Node>) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::AddNode,
            body: Body::AddNode { nodes },
        }
    }

    pub fn state_change(mode: Mode) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::StateChange,
            body: Body::StateChange { mode },
        }
    }

    pub fn ping() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::Ping,
            body: Body::Empty,
        }
    }

    pub fn pong() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Pong,
            body: Body::Empty,
        }
    }

    pub fn ack(message_id: u64) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: true,
            parcel_type: Type::Ack,
            body: Body::Ack { message_id },
        }
    }
    pub fn resource_request(resource_request: ResourceRequest) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRequest,
            body: Body::ResourceRequest {
                resource_request
            },
        }
    }
    pub fn resource_release(resource_release: ResourceRelease) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRelease,
            body: Body::ResourceRelease {
                resource_release
            },
        }
    }

    pub fn ext_addr_req() -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRelease,
            body: Body::Empty,
        }
    }
    pub fn ext_addr_res(addr: SocketAddr) -> ProtoParcel {
        ProtoParcel {
            id: generate_id(),
            proto_version: PROTO_VERSION.clone(),
            sender_id: *SENDER.lock().unwrap(),
            is_response: false,
            parcel_type: Type::ResourceRelease,
            body: Body::ExtAddrRes { addr },
        }
    }
    pub fn proto_error() -> ProtoParcel {
        ProtoParcel {
            proto_version: PROTO_VERSION.clone(),
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