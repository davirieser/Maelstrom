use serde::{Deserialize, Serialize};

use crate::types::message::Message;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Packet {
    pub src: String,
    pub dest: String,
    pub body: Message,
}
