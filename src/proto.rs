use bytes::{BytesMut, BufMut, Buf};
use std::convert::{TryFrom};
use std::{mem, fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::Node;
use crate::proto::Body::DSQREQ;
use std::fmt::Display;
use num_traits::{FromPrimitive, ToPrimitive};
use crate::proto::Type::DSCRES;
use serde::{Serialize, Deserialize};

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    DSCREQ = 1,
    DSCRES = 2,
    SEQREQ = 3,
    SEQRES = 4,
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::DSCREQ => write!(f, "{}", "DSCREQ"),
            Type::DSCRES => write!(f, "{}", "DSCRES"),
            Type::SEQREQ => write!(f, "{}", "SEQREQ"),
            Type::SEQRES => write!(f, "{}", "SEQRES"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Header {
    pub id: u16,
    pub size: u16,
    pub meta: u16,
    pub parcel_type: Type, // u16
}

impl Header {
    pub fn new(id: u16, size: u16, meta: u16, parcel_type: Type) -> Self {
        Header { id, size, meta, parcel_type }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Body {
    DSQREQ { identity: Node },
    DSQRES {
        size: u16,
        neighbours: Vec<Node>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct ProtoParcel {
    pub header: Header,
    pub body: Body,
}