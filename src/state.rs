pub struct State {
    pub name: String,
    pub mode: Mode,
    pub neighbours: Vec<String>,
}

impl State {
    pub fn new(name: String, mode: Mode, neighbours: Vec<String>) -> State {
        State { name, mode, neighbours }
    }
    pub fn change_mode(&mut self, mode: Mode) {
        self.mode = mode;
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