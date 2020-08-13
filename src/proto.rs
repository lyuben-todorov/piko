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
}

pub struct Body {
    pub size: u16,
    pub bytes: Bytes,
}