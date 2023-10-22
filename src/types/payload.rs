use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::packet::Packet;

#[derive(Debug, Serialize, Deserialize)]
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
    SyncRequest {
        start: usize,
        end: usize,
    },
    Batch {
        packets: Vec<Packet>,
    },
    Ack {
        messages: Vec<usize>,
    },
    Forward {
        packet: Box<Packet>,
    },
}
