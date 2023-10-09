use crate::types::{
    message::Message,
    message_handler::MessageHandler,
    packet::Packet,
    packet_handler::{MessageResponse, NodeInfo},
    payload::Payload,
};

pub struct TopologyHandler {}

impl MessageHandler for TopologyHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        _state: &NodeInfo,
        send: &mut dyn FnMut(MessageResponse),
    ) {
        if let Packet {
            src,
            body:
                Message {
                    msg_id,
                    payload: Payload::Topology { topology },
                    ..
                },
            ..
        } = packet
        {
            send(MessageResponse {
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::TopologyOk {},
            });
        }
    }
}
