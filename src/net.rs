use std::net::TcpStream;
use crate::proto::{Header, Type};
use crate::state::{State, Node};
use std::io::{Write, Read};
use bytes::{BytesMut, Buf};
use std::convert::{TryInto, TryFrom};
use bytes::buf::BufExt;

// Manages connection between host node and one of its neighbours during DSC phase.
pub struct DscConnection<'a> {
    pub conn: TcpStream,
    pub node_state: &'a State,
}

impl<'a> DscConnection<'a> {
    pub fn new(conn: TcpStream, node_state: &'a State) -> Self {
        DscConnection { conn, node_state }
    }

    pub fn handshake(&mut self) -> Result<Node, String> {
        let size: u16 = 8;

        let req_header = Header::new(self.node_state.self_node.id, size, 0, Type::DSCREQ);

        let mut req_bytes = BytesMut::with_capacity(size as usize);

        let encoded_header: Vec<u8> = req_header.try_into().unwrap();
        req_bytes.extend(encoded_header.as_slice());

        // add message here

        let encoded_node = Vec::try_from(&self.node_state.self_node);

        let write_result = self.conn.write(req_bytes.bytes());
        println!("Written {} bytes to stream.", write_result.unwrap());

        // expect DSCRES on same connection
        let mut res = Vec::<u8>::new();

        let res_size = self.conn.read_to_end(&mut res);
        let mut res = BytesMut::from(res.as_slice());

        let header = res.split_to(8).to_vec();
        let header: Header = Header::try_from(&header)?;

        let node_byte_size = res.get_u16();
        let node = res.split_to(node_byte_size as usize);
        let node = Node::try_from(&node.to_vec()).unwrap();

        if header.parcel_type == Type::DSCRES {
            Ok(node)
        } else {
            Err(format!("Expected DSCRES, found {}", header.parcel_type))
        }
    }
}