use bytes::{BytesMut, BufMut, Buf};
use std::convert::{TryFrom};
use std::{mem, fmt};
use num_derive::{FromPrimitive, ToPrimitive};
use crate::state::Node;
use crate::proto::Body::DSQREQ;
use std::fmt::Display;
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, Debug, PartialEq)]
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

impl TryFrom<&Vec<u8>> for Header {
    type Error = String;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::from(value.as_slice());


        if value.len() != mem::size_of::<Header>() {
            return Err(format!("Mismatched header sizes! Got {}", value.len()));
        }

        let id = bytes.get_u16();
        let size = bytes.get_u16();
        let meta = bytes.get_u16();
        let parcel_type = bytes.get_u16();
        let parcel_type: Type = match num::FromPrimitive::from_u16(parcel_type) {
            Some(parcel_type) => parcel_type,
            None => return Err(format!("Missmatching parcel type! Got {}", parcel_type))
        };

        Ok(Header { id, size, meta, parcel_type })
    }
}

impl TryFrom<Header> for Vec<u8> {
    type Error = &'static str;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::with_capacity(8);
        bytes.put_u16(value.id);
        bytes.put_u16(value.size);
        bytes.put_u16(value.meta);
        bytes.put_u16(num::ToPrimitive::to_u16(&value.parcel_type).unwrap());

        Ok(bytes.to_vec())
    }
}

pub enum Body {
    DSQREQ { identity: Node },
    DSQRES { neighbours: Vec<Node> },
}

impl TryFrom<&Vec<u8>> for Body {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::from(value.as_slice());

        let parcel_type: Type = match num::FromPrimitive::from_u16(bytes.get_u16()) {
            Some(parcel_type) => parcel_type,
            None => return Err("Mismatching parcel type!")
        };

        match parcel_type {
            Type::DSCREQ => {
                return Ok(DSQREQ { identity: Node::try_from(&bytes.to_vec()).unwrap() });
            }
            Type::DSCRES => {}
            Type::SEQREQ => {}
            Type::SEQRES => {}
        }

        Err("This shouldn't be happening")
    }
}

impl TryFrom<Body> for Vec<u8> {
    type Error = &'static str;

    fn try_from(value: Body) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}


pub struct ProtoParcel {
    pub header: Header,
    pub body: Body,
}