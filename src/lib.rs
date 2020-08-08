pub mod discovery;

#[derive(Clone, Copy)]
pub enum State {
    WRK,
    DSC,
    ERR,
    PANIC,
    SHUTDOWN,
}

pub fn state_to_str(state: &State) -> &'static str {
    match state {
        State::WRK=> "WRK",
        State::DSC=> "DSC",
        State::ERR=>"ERR",
        State::PANIC=>"PANIC",
        State::SHUTDOWN=> "SHUTDOWN"
    }
}