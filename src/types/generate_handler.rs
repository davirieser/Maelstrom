use crate::types::{
    message::Message, message_handler::MessageHandler, node_info::NodeInfo, packet::Packet,
    message_response::MessageResponse, payload::Payload,
};

pub struct GenerateHandler {}

impl MessageHandler for GenerateHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        _state: &NodeInfo,
    ) -> Option<Vec<MessageResponse>> {
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
            Some(vec![MessageResponse::NoAck {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::GenerateOk {
                    id: _state.msg_number * _state.node_ids.len() + _state.node_number,
                },
            }])
        } else {
            None
        }
    }
}
