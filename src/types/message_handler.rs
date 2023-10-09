use crate::types::{packet::Packet, packet_handler::MessageResponse, packet_handler::NodeInfo};

pub trait MessageHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        state: &NodeInfo,
        send: &mut dyn FnMut(MessageResponse),
    );
}
