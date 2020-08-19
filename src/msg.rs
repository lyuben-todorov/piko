use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    DSCREQ {},
     DSCRES {},
    SEQREQ {},
    SEQRES {},

}