use std::net::TcpStream;

pub struct DscConnection {
    pub conn: TcpStream
}

impl DscConnection {
    pub fn new(conn: TcpStream) -> Self {
        DscConnection { conn }
    }

    pub fn handshake() -> bool {
        true
    }
}