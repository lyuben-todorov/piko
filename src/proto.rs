use std::{fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::{Node, Mode};

use std::fmt::Display;

use serde::{Serialize, Deserialize};

static PROTO_VERSION: &str = "1.0";

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
}

// Enumeration over the types of protocol errors.
#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProtoError {
    BadRes = 1,
    BadReq = 2,

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
            Type::Pong => write!(f, "{}", "Pong")
        }
    }
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
}


#[derive(Serialize, Deserialize)]
pub struct ProtoParcel {
    // whether or not packet is a response
    pub is_response: bool,
    // type of packet
    pub parcel_type: Type,
    // id of sender node
    pub id: u16,
    // size of application-specific data in bytes
    pub size: u16,
    // message body
    pub body: Body,
}

impl ProtoParcel {
    pub fn dsc_req(self_node_information: Node) -> ProtoParcel {
        ProtoParcel { is_response: false, parcel_type: Type::DscReq, id: self_node_information.id, size: 0, body: Body::DscReq { identity: self_node_information } }
    }
    pub fn dsc_res(id: u16, neighbours_information: Vec<Node>) -> ProtoParcel {
        ProtoParcel { is_response: true, parcel_type: Type::DscRes, id, size: 0, body: Body::DscRes { neighbours: neighbours_information } }
    }
    pub fn seq_req(id: u16) -> ProtoParcel {
        ProtoParcel { is_response: false, parcel_type: Type::SeqReq, id, size: 0, body: Body::Empty }
    }
    pub fn seq_res(id: u16, seq_number: u8) -> ProtoParcel {
        ProtoParcel { is_response: true, parcel_type: Type::SeqRes, id, size: 0, body: Body::SeqRes { seq_number } }
    }
    pub fn state_change(id: u16, mode: Mode) -> ProtoParcel {
        ProtoParcel { is_response: false, parcel_type: Type::StateChange, id, size: 0, body: Body::StateChange { mode } }
    }
    pub fn ping(id: u16) -> ProtoParcel {
        ProtoParcel { is_response: false, parcel_type: Type::Ping, id, size: 0, body: Body::Empty }
    }
    pub fn pong(id: u16) -> ProtoParcel {
        ProtoParcel { is_response: true, parcel_type: Type::Pong, id, size: 0, body: Body::Empty }
    }
}
