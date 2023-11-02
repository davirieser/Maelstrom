use crate::types::{
    collection::Collection, message::Message, message_handler::MessageHandler,
    message_response::MessageResponse, node_info::NodeInfo, packet::Packet, payload::Payload,
};

pub struct EchoHandler {}

impl MessageHandler for EchoHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        _state: &NodeInfo,
    ) -> Collection<MessageResponse> {
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
            Collection::One(MessageResponse::NoAck {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::EchoOk { echo: echo.clone() },
            })
        } else {
            Collection::None
        }
    }
}
