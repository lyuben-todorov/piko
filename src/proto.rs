use bit_vec::BitVec;
use std::ops::Deref;
use bytes::{BytesMut, BufMut, Bytes, Buf};

pub fn decode_header(header: &mut Bytes) -> Header {
    assert_eq!(header.len(), 8);
    let meta = header.get_u16();
    let size = header.get_u16();
    let id = header.get_u32();


    return Header { id, size, meta };
}

pub fn encode_header(id: u32, size: u16, meta: u16) -> Bytes {
    let mut buf = BytesMut::new();
    buf.put_u32(id);
    buf.put_u16(size);
    buf.put_u16(meta);

    buf.freeze()
}

pub struct Header {
    pub id: u32,
    pub size: u16,
    pub meta: u16,
}

impl Header {
    pub fn new(id: u32, size: u16, meta: u16) -> Self {
        Header { id, size, meta }
    }
    pub fn encode_header(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u32(self.id);
        buf.put_u16(self.size);
        buf.put_u16(self.meta);
        buf.freeze()
    }
}


pub struct Body {
    pub size: u16,
    pub message: String,
}

impl Body {
    pub fn new(message: String) -> Self {
        Body { size: (message.len()) as u16, message }
    }

    pub fn encode_body(&self) -> Bytes {
        let mut buf = BytesMut::new();
        buf.put_u16(self.size);
        buf.put_slice(self.message.as_bytes());
        buf.freeze()
    }
}

pub struct Message {
    pub header: Header,
    pub body: Body,
}

impl Message {

}

