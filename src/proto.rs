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

pub fn encode_header(buf: &mut BytesMut, id: u32, size: u16, meta: u16) {
    buf.put_u32(id);
    buf.put_u16(size);
    buf.put_u16(meta);
}

pub struct Header {
    pub id: u32,
    pub size: u16,
    pub meta: u16,
}