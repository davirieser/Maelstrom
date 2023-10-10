use crate::types::{
    message::Message,
    message_handler::MessageHandler,
    packet::Packet,
    packet_handler::{MessageResponse, NodeInfo},
    payload::Payload,
};

pub struct EchoHandler {}

impl MessageHandler for EchoHandler {
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
                    payload: Payload::Echo { echo },
                    ..
                },
            ..
        } = packet
        {
            send(MessageResponse {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::EchoOk { echo: echo.clone() },
            });
        }
    }
}
