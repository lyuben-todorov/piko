use bit_vec::BitVec;
use std::ops::Deref;
use bytes::{BytesMut, BufMut, Bytes, Buf};
use std::convert::{TryFrom, TryInto};

pub struct Header {
    pub id: u32,
    pub size: u16,
    pub meta: u16,
}

impl Header {
    pub fn new(id: u32, size: u16, meta: u16) -> Self {
        Header { id, size, meta }
    }
}

impl TryFrom<&Vec<u8>> for Header {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::from(value.as_slice());

        if value.len() != 8 {
            return Err("Mismatched header sizes!");
        }

        let meta = bytes.get_u16();
        let size = bytes.get_u16();
        let id = bytes.get_u32();

        Ok(Header { id, size, meta })
    }
}

impl TryFrom<Header> for Vec<u8> {
    type Error = &'static str;

    fn try_from(value: Header) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::with_capacity(8);
        bytes.put_u32(value.id);
        bytes.put_u16(value.size);
        bytes.put_u16(value.meta);

        Ok(bytes.to_vec())

    }
}

pub struct Body {
    pub size: u16,
    pub message: String,
}

impl TryFrom<&[u8]> for Body {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::from(value);

        let declared_size = bytes.get_u16();

        let message = String::from_utf8(bytes.to_vec()).unwrap();

        let message_size: u16 = u16::try_from(message.len()).unwrap();

        if message_size == declared_size {
            Ok(Body { size: message_size, message })
        } else {
            Err("Mismatched message sizes")
        }
    }
}

impl Body {
    pub fn new(message: String) -> Self {
        Body { size: (message.len()) as u16, message }
    }
}

pub struct Message {
    pub header: Header,
    pub body: Body,
}

impl Message {}

