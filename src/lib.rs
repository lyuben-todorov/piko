pub mod discovery;

pub enum State {
    WRK,
    DSC,
    ERR,
    PANIC,
    SHUTDOWN,
}