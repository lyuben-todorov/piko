

use std::{fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::Node;

use std::fmt::Display;

use serde::{Serialize, Deserialize};

static PROTO_VERSION: &str = "1.0";

// Enumeration over the types of protocol messages
#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    DscReq = 1,
    DscRes = 2,
    SeqReq = 3,
    SeqRes = 4,

    Err = 5,
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
            Type::Err => write!(f, "{}", "Err")
        }
    }
}


#[derive(Serialize, Deserialize)]
pub enum Body {
    DscReq {
        identity: Node
    },
    DscRes {
        neighbours: Vec<Node>,
    },
}


#[derive(Serialize, Deserialize)]
pub struct ProtoParcel<'a> {
    pub proto_version: &'a str,
    pub is_response: bool,
    // whether or not packet is a response
    pub parcel_type: Type,
    // type of packet
    pub id: u16,
    // id of sender node
    pub size: u16, // size of body in bytes

    pub body: Body,
}

impl<'a> ProtoParcel<'a> {
    pub fn dsq_req(self_node: &Node) -> ProtoParcel {
        let node: Node = self_node.clone();
        ProtoParcel { proto_version: PROTO_VERSION, is_response: false, parcel_type: Type::DscReq, id: self_node.id, size: 0, body: Body::DscReq { identity: node } }
    }
}
