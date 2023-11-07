use crate::types::message::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Packet {
    pub src: String,
    pub dest: String,
    pub body: Message,
}
