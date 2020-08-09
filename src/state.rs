use std::collections::{HashMap, HashSet};

pub struct State {
    pub name: String,
    pub mode: Mode,
    pub neighbours: HashMap<String, Node>,
}

impl State {
    pub fn new(name: String, mode: Mode, neighbours: HashMap<String, Node>) -> Self {
        State { name, mode, neighbours }
    }

    fn add_neighbour(&mut self, name: &str, node: Node) {
        self.neighbours.insert(name.parse().unwrap(), node);
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

pub struct Node {
    pub name: String,
    pub mode: Mode,
    pub host: String,
}

impl Node {
    pub fn new(name: String, mode: Mode, host: String) -> Node {
        Node { name, mode, host }
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