use bytes::{BytesMut, BufMut, Buf};
use std::convert::{TryFrom};
use std::{mem, fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::Node;
use crate::proto::Body::DSQREQ;
use std::fmt::Display;
use num_traits::{FromPrimitive, ToPrimitive};
use crate::proto::Type::DscRes;
use serde::{Serialize, Deserialize};

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    DscReq = 1,
    DscRes = 2,
    SeqReq = 3,
    SeqRes = 4,
}

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProtoError {
    BadRes = 1,
    BadReq = 2,

}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::DscReq => write!(f, "{}", "DSCREQ"),
            Type::DscRes => write!(f, "{}", "DSCRES"),
            Type::SeqReq => write!(f, "{}", "SEQREQ"),
            Type::SeqRes => write!(f, "{}", "SEQRES"),
        }
    }
}


#[derive(Serialize, Deserialize)]
pub enum Body {
    DSQREQ {
        identity: Node
    },
    DSQRES {
        neighbours: Vec<Node>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct ProtoParcel {
    pub proto_version: u16,
    pub is_response: bool,
    // whether or not packet is a response
    pub parcel_type: Type,
    // type of packet
    pub id: u16,
    // id of sender node
    pub size: u16, // size of body in bytes

    pub body: Body,
}
