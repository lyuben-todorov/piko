use std::collections::{HashMap};
use sha2::{Sha256, Digest};
use byteorder::{ReadBytesExt, BigEndian};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::export::TryFrom;
use bytes::{BytesMut, BufMut, Buf};

#[derive(FromPrimitive, ToPrimitive)]
pub enum Mode {
    WRK = 1,
    DSC = 2,
    ERR = 3,
    PANIC = 4,
    SHUTDOWN = 5,
}

pub struct State {
    pub name: String,
    pub mode: Mode,
    pub id: u16,
    pub neighbours: HashMap<String, Node>,
    pub cluster_size: usize,
}

impl State {
    pub fn new(name: String, mode: Mode, neighbours: HashMap<String, Node>) -> Self {
        let id = Sha256::digest(name.as_bytes()).as_slice().read_u16::<BigEndian>().unwrap();

        State { name, mode, id, neighbours, cluster_size: 0 }
    }

    pub fn add_neighbour(&mut self, name: &str, node: Node) {
        self.neighbours.insert(name.parse().unwrap(), node);
        self.cluster_size += 1;
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

pub struct Node {
    pub id: u16,
    pub mode: Mode,
    //u16
    pub name: String,
    pub host: String,
}

impl TryFrom<&Vec<u8>> for Node {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::from(value.as_slice());

        let host_size = bytes.get_u8();
        let host = String::from_utf8(bytes.split_to(host_size as usize).to_vec()).unwrap();
        let name_size = bytes.get_u8();
        let name = String::from_utf8(bytes.split_to(name_size as usize).to_vec()).unwrap();
        let mode: Mode = match num::FromPrimitive::from_u16(bytes.get_u16()) {
            Some(mode) => mode,
            None => return Err("Mismatching parcel type!")
        };
        let id = bytes.get_u16();

        Ok(Node { id, mode, name, host })
    }
}

impl TryFrom<Node> for Vec<u8> {
    type Error = &'static str;

    fn try_from(value: Node) -> Result<Self, Self::Error> {
        let mut bytes = BytesMut::new();

        bytes.put_u16(value.id);
        bytes.put_u16(num::ToPrimitive::to_u16(&value.mode).unwrap());
        bytes.put(value.name.as_bytes());
        bytes.put_u8(value.name.len() as u8);
        bytes.put(value.host.as_bytes());
        bytes.put_u8(value.name.len() as u8);

        Ok(bytes.to_vec())
    }
}

impl Node {
    pub fn new(name: String, mode: Mode, host: String, id: u16) -> Node {
        Node { name, mode, host, id }
    }
}

