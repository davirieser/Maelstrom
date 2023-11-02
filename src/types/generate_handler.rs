use crate::types::{
    collection::Collection, message::Message, message_handler::MessageHandler,
    message_response::MessageResponse, node_info::NodeInfo, packet::Packet, payload::Payload,
};

pub struct GenerateHandler {
    pub counter: usize
}

impl MessageHandler for GenerateHandler {
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
                    payload: Payload::Generate,
                    ..
                },
            ..
        } = packet
        {
            self.counter += 1;
            Collection::One(MessageResponse::NoAck {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::GenerateOk {
                    id: self.counter * _state.node_ids.len() + _state.node_number,
                },
            })
        } else {
            Collection::None
        }
    }
}
