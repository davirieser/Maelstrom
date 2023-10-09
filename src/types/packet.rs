use serde::{Deserialize, Serialize};

use crate::types::message::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Packet {
    pub src: String,
    pub dest: String,
    pub body: Message,
}
