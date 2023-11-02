use crate::types::message::Message;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

#[derive(Debug)]
pub struct NodeInfo {
    /// The Node Id of this Node.
    pub node_id: String,
    /// All Node Ids of Client Nodes in the network.
    pub client_nodes: Vec<String>,
    /// All Node Ids of Server Nodes in the network.
    pub server_nodes: Vec<String>,
    /// All Node Ids in the Network (Server and Client Nodes).
    pub node_ids: Vec<String>,
    /// The Topology given by Maelstrom.
    pub topology: HashMap<String, Vec<String>>,
    /// Efficient Topology for sending Broadcast through network.
    /// Use the "src" Address of the Broadcast Packet to get the Node Ids that the broadcast should
    /// be forwarded to from this Node.
    pub broadcast_topology: HashMap<String, Vec<String>>,
    /// The number of this node.
    pub node_number: usize,
    /// Internal number used for generating "msg_id" for packets (only increment).
    pub msg_number: usize,
    pub conn_info: HashMap<String, NodeConnectionInfo>,
}

#[derive(Debug)]
pub struct NodeConnectionInfo {
    pub out_msg_id: usize,
    pub in_msg_id: MessageSyncStatus,
    pub un_ack_messages: Vec<Message>,
}

#[derive(Debug)]
pub enum MessageSyncStatus {
    Synced {
        last_msg_id: usize,
    },
    NotSynced {
        last_msg_id: usize,
        missing_msg_ids: HashSet<usize>,
    },
}

// TODO: Change the function names to better represent their behaviour.
impl MessageSyncStatus {
    pub fn is_next_msg_id(&self, msg_id: usize) -> Ordering {
        match self {
            Self::Synced { last_msg_id } | Self::NotSynced { last_msg_id, .. } => {
                last_msg_id.cmp(&msg_id)
            }
        }
    }
    pub fn get_next_msg_id(&self) -> usize {
        match self {
            Self::Synced { last_msg_id } | Self::NotSynced { last_msg_id, .. } => last_msg_id + 1,
        }
    }
    pub fn valid_message(&self, msg_id: usize) -> bool {
        match self {
            Self::Synced { last_msg_id } => *last_msg_id == msg_id,
            Self::NotSynced {
                last_msg_id,
                missing_msg_ids,
            } => *last_msg_id == msg_id || missing_msg_ids.contains(&msg_id),
        }
    }
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced { .. })
    }
    pub fn increment_msg_id(&mut self) {
        match self {
            Self::Synced { last_msg_id } | Self::NotSynced { last_msg_id, .. } => *last_msg_id += 1,
        }
    }
    pub fn add_missing_msg_ids(&mut self, msg_id: usize) {
        match self {
            Self::Synced { last_msg_id } => {
                if *last_msg_id < msg_id {
                    *self = Self::NotSynced {
                        last_msg_id: msg_id,
                        missing_msg_ids: (*last_msg_id..msg_id).collect(),
                    }
                }
            }
            Self::NotSynced {
                last_msg_id,
                missing_msg_ids,
            } => {
                if *last_msg_id < msg_id {
                    *last_msg_id = msg_id;
                    missing_msg_ids.extend(*last_msg_id..msg_id);
                }
            }
        }
    }
}
