use crate::types::{message_response::MessageResponse, node_info::NodeInfo, packet::Packet};

pub trait MessageHandler {
    fn handle_message(&mut self, packet: &Packet, state: &NodeInfo)
        -> Option<Vec<MessageResponse>>;
}
