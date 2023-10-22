use crate::types::packet::Packet;
use std::collections::HashMap;

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
    pub in_msg_id: usize,
    pub un_ack_messages: Vec<Packet>,
}
