use crate::types::{
    message::Message,
    message_handler::MessageHandler,
    packet::Packet,
    packet_handler::{MessageResponse, NodeInfo},
    payload::Payload,
};

pub struct GenerateHandler {}

impl MessageHandler for GenerateHandler {
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
                    payload: Payload::Generate,
                    ..
                },
            ..
        } = packet
        {
            send(MessageResponse {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::GenerateOk {
                    id: _state.msg_number * _state.node_ids.len() + _state.node_number,
                },
            });
        }
    }
}
