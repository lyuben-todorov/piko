use std::collections::{HashMap};
use sha2::{Sha256, Digest};
use byteorder::{ReadBytesExt, BigEndian};
use num_derive::{FromPrimitive, ToPrimitive};


use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::fmt::Display;
use serde::export::Formatter;
use std::fmt;


#[derive(FromPrimitive, ToPrimitive, Deserialize, Serialize, Clone)]
pub enum Mode {
    WRK = 1,
    DSC = 2,
    ERR = 3,
    PANIC = 4,
    SHUTDOWN = 5,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Mode::WRK => write!(f, "{}", "Wrk"),
            Mode::DSC => write!(f, "{}", "Dsc"),
            Mode::ERR => write!(f, "{}", "Err"),
            Mode::PANIC => write!(f, "{}", "Panic"),
            Mode::SHUTDOWN => write!(f, "{}", "Shutdown"),
        }
    }
}

pub struct State {
    pub self_node_information: Node,
    pub neighbours: HashMap<String, Node>,

}

impl State {
    pub fn new(self_node_information: Node, neighbours: HashMap<String, Node>) -> Self {
        State { self_node_information, neighbours }
    }

    pub fn get_size(&self) -> usize {
        return self.neighbours.len();
    }

    pub fn add_neighbour(&mut self, node: Node) {
        self.neighbours.insert(node.name.clone(), node);
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.self_node_information.mode = mode;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: u16,
    pub mode: Mode,
    pub name: String,
    pub host: String,
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
    pub fn new(name: String, mode: Mode, host: String) -> Node {
        let id = Sha256::digest(name.as_bytes()).as_slice().read_u16::<BigEndian>().unwrap();

        Node { name, mode, host, id }
    }
}

