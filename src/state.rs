use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use byteorder::{ReadBytesExt, BigEndian};
use num_derive::{FromPrimitive, ToPrimitive};


use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::fmt::Display;
use serde::export::Formatter;
use std::fmt;
use std::net::SocketAddr;
use crate::state::Mode::Wrk;


#[derive(FromPrimitive, ToPrimitive, Deserialize, Serialize, Clone, PartialEq)]
pub enum Mode {
    Wrk = 1,
    Dsc = 2,
    Err = 3,
    Panic = 4,
    Shutdown = 5,
    TimedOut = 6,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Wrk => write!(f, "{}", "Wrk"),
            Mode::Dsc => write!(f, "{}", "Dsc"),
            Mode::Err => write!(f, "{}", "Err"),
            Mode::Panic => write!(f, "{}", "Panic"),
            Mode::Shutdown => write!(f, "{}", "Shutdown"),
            Mode::TimedOut => write!(f, "{}", "TimedOut")
        }
    }
}

pub struct State {
    pub self_node_information: Node,
    pub neighbours: HashMap<u16, Node>,
    pub sequence: u8,
}

impl State {
    pub fn new(self_node_information: Node, neighbours: HashMap<u16, Node>) -> Self {
        State { self_node_information, neighbours, sequence: 0 }
    }

    pub fn get_cluster_size(&self) -> usize {
        return self.neighbours.len();
    }

    pub fn add_neighbour(&mut self, node: Node) {
        self.neighbours.insert(node.id, node);
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.self_node_information.mode = mode;
    }

    pub fn get_neighbour_addrs(&self) -> Vec<SocketAddr> {
        let neighbour_list: Vec<Node> = self.get_active_neighbours();

        let neighbour_list: Vec<SocketAddr> = neighbour_list.iter().map(|node| { node.host }).collect(); // get socketaddrs

        neighbour_list
    }

    pub fn get_neighbour_keys(&self) -> HashSet<u16> {
        self.neighbours.keys().cloned().collect()
    }

    pub fn get_active_neighbours(&self) -> Vec<Node> {
        self.neighbours.values().filter(|val| val.mode == Wrk).cloned().collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: u16,
    pub mode: Mode,
    pub name: String,
    pub host: SocketAddr,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl Node {
    pub fn new(name: String, mode: Mode, host: SocketAddr) -> Node {
        let id = Sha256::digest(name.as_bytes()).as_slice().read_u16::<BigEndian>().unwrap();

        Node { name, mode, host, id }
    }
}

