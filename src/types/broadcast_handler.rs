use crate::types::{
    message::Message,
    message_handler::MessageHandler,
    packet::Packet,
    packet_handler::{MessageResponse, NodeInfo},
    payload::Payload,
};

pub struct BroadcastHandler {
    pub messages: Vec<usize>,
}

impl MessageHandler for BroadcastHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        state: &NodeInfo,
        send: &mut dyn FnMut(MessageResponse),
    ) {
        match packet {
            Packet {
                src,
                body:
                    Message {
                        msg_id,
                        payload: Payload::Broadcast { message },
                        ..
                    },
                ..
            } => {
                self.messages.push(*message);
                send(MessageResponse {
                    dest: src.clone(),
                    in_reply_to: *msg_id,
                    payload: Payload::BroadcastOk,
                });
                for server in state.broadcast_topology[&state.node_id].iter() {
                    send(MessageResponse {
                        dest: state.node_id.clone(),
                        in_reply_to: Option::None,
                        payload: Payload::BroadcastIntern { message: *message },
                    });
                }
            }
            Packet {
                src,
                body:
                    Message {
                        payload: Payload::BroadcastIntern { message },
                        ..
                    },
                ..
            } => {
                self.messages.push(*message);
                // TODO: Send Internal Broadcast to other nodes in network
                // NOTE: You have to set the "src" to the "src" of the incoming packet, not to this
                // node's id. Probably have to add an Optional<String> to MessageResponse.
            }
            Packet {
                src,
                body:
                    Message {
                        msg_id,
                        payload: Payload::Read,
                        ..
                    },
                ..
            } => {
                send(MessageResponse {
                    dest: src.clone(),
                    in_reply_to: *msg_id,
                    payload: Payload::ReadOk {
                        messages: self.messages.clone(),
                    },
                });
            }
            _ => {}
        }
    }
}
