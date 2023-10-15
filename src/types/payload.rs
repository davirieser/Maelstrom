use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{message::Message, packet::Packet};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        id: usize,
    },
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    SyncRequest {
        start: usize,
        end: usize,
    },
    Batch {
        messages: Vec<Message>,
    },
    Ack,
    Forward {
        packet: Box<Packet>,
    },
    Error {
        code: usize,
        text: String,
    },
}
