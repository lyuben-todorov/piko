use std::collections::{HashMap};
use sha2::{Sha256, Digest};
use byteorder::{ReadBytesExt, BigEndian};
use num_derive::{FromPrimitive, ToPrimitive};
use serde::export::TryFrom;
use bytes::{BytesMut, BufMut, Buf};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Serialize, Deserialize};

#[derive(FromPrimitive, ToPrimitive, Deserialize, Serialize)]
pub enum Mode {
    WRK = 1,
    DSC = 2,
    ERR = 3,
    PANIC = 4,
    SHUTDOWN = 5,
}

pub struct State {
    pub self_node: Node,
    pub neighbours: HashMap<String, Node>,
    pub cluster_size: usize,
}

impl State {
    pub fn new(self_node: Node, neighbours: HashMap<String, Node>) -> Self {
        State { self_node, neighbours, cluster_size: 0 }
    }

    pub fn add_neighbour(&mut self, name: &str, node: Node) {
        self.neighbours.insert(name.parse().unwrap(), node);
        self.cluster_size += 1;
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.self_node.mode = mode;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: u16,
    pub mode: Mode,
    //u16
    pub name: String,
    pub host: String,
}

impl Node {
    pub fn new(name: String, mode: Mode, host: String) -> Node {
        let id = Sha256::digest(name.as_bytes()).as_slice().read_u16::<BigEndian>().unwrap();

        Node { name, mode, host, id }
    }
}

