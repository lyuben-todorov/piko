use std::net::{TcpStream, TcpListener};
use crate::proto::{Type, ProtoParcel, ProtoError};
use crate::state::{State, Node};
use std::io::{Write, Read};
use bytes::{BytesMut, Buf};
use std::convert::{TryInto, TryFrom};
use bytes::buf::BufExt;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

pub fn listener_thread(rx: Receiver<u32>, tx: Sender<u32>, state: Arc<RwLock<State>>, socket: TcpListener) {}
