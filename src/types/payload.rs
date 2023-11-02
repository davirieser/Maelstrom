use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{message::Message, packet::Packet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    // NOTE: Standard Payloads
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
    Error {
        code: usize,
        text: String,
    },
    // NOTE: Custom Payloads
    SyncRequest,
    Batch {
        messages: Vec<Message>,
    },
    Ack,
    MultiAck {
        messages: Vec<usize>,
    },
    Forward {
        packet: Box<Packet>,
    },
}
