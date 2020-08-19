use std::net::TcpStream;
use crate::proto::{Body, Header};
use crate::state::{State, Node};
use std::io::{Write, Read};
use bytes::{BytesMut, Buf};
use byteorder::ReadBytesExt;
use std::error::Error;

// Manages connection between host node and one of its neighbours during DSC phase.
pub struct DscConnection<'a> {
    pub conn: TcpStream,
    pub node_state: &'a State,
}

impl<'a> DscConnection<'a> {
    pub fn new(conn: TcpStream, node_state: &'a State) -> Self {
        DscConnection { conn, node_state }
    }

    pub fn handshake(&mut self) -> Result<Error, Node> {
        let size: u16 = 8;
        let req_header = Header::new(self.node_state.id, size, 0);

        let mut req_bytes = BytesMut::with_capacity(size as usize);

        req_bytes.extend(req_header.encode_header());

        // add message here

        self.conn.write(req_bytes.bytes());

        // parse response
        let mut res = Vec::<u8>::new();
        self.conn.read_to_end(&mut res);

        // return handshake result
        true
    }
}