use std::collections::{HashMap};
use sha2::{Sha256, Digest};
use byteorder::{ReadBytesExt, BigEndian};

pub struct State {
    pub name: String,
    pub mode: Mode,
    pub id: u32,
    pub neighbours: HashMap<String, Node>,
    pub cluster_size: usize,
}

impl State {
    pub fn new(name: String, mode: Mode, neighbours: HashMap<String, Node>) -> Self {
        let id = Sha256::digest(name.as_bytes()).as_slice().read_u32::<BigEndian>().unwrap();

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
    pub name: String,
    pub mode: Mode,
    pub id: u32,
    pub host: String,
}

impl Node {
    pub fn new(name: String, mode: Mode, host: String, id: u32) -> Node {
        Node { name, mode, host, id }
    }
}

#[derive(Clone, Copy)]
pub enum Mode {
    WRK,
    DSC,
    ERR,
    PANIC,
    SHUTDOWN,
}

pub fn state_to_str(state: &Mode) -> &'static str {
    match state {
        Mode::WRK => "WRK",
        Mode::DSC => "DSC",
        Mode::ERR => "ERR",
        Mode::PANIC => "PANIC",
        Mode::SHUTDOWN => "SHUTDOWN"
    }
}