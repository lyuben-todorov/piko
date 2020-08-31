use std::net::{TcpStream, TcpListener};
use crate::proto::{Type, ProtoParcel, ProtoError};
use crate::state::{State, Node};
use std::io::{Write, Read};
use bytes::{BytesMut, Buf};
use std::convert::{TryInto, TryFrom};
use bytes::buf::BufExt;
use crate::proto::Body::{DSQREQ, DSQRES};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, Arc};

pub fn net_thread(rx: Receiver<u32>, state: Arc<Mutex<State>>, socket: TcpListener) {

}

// Dispatch handshake
pub fn handshake(hostname: &String, node_data: &Node) -> Result<Node, ProtoError> {}
