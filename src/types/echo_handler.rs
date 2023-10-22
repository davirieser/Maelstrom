use crate::types::{
    message::Message, message_handler::MessageHandler, node_info::NodeInfo, packet::Packet,
    message_response::MessageResponse, payload::Payload,
};

pub struct EchoHandler {}

impl MessageHandler for EchoHandler {
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
                    payload: Payload::Echo { echo },
                    ..
                },
            ..
        } = packet
        {
            Some(vec![MessageResponse::NoAck {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::EchoOk { echo: echo.clone() },
            }])
        } else {
            None 
        }
    }
}
