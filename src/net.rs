use std::net::TcpStream;
use crate::proto::{Body, Header};
use crate::state::State;

// Manages connection between host node and one of its neighbours during DSC phase.
pub struct DscConnection<'a> {
    pub conn: TcpStream,
    pub node_state: &'a State,
}

impl<'a> DscConnection<'a> {
    pub fn new(conn: TcpStream, node_state: &'a State) -> Self {
        DscConnection { conn, node_state }
    }

    pub fn handshake(&mut self) -> bool {
        let size: u16 = 0;
        let req_header = Header::new(self.node_state.id, size, 0);
        true
    }
}