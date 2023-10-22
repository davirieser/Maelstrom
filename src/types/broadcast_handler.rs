use crate::types::{
    message::Message, message_handler::MessageHandler, message_response::MessageResponse,
    node_info::NodeInfo, packet::Packet, payload::Payload,
};

pub struct BroadcastHandler {
    pub messages: Vec<usize>,
}

impl MessageHandler for BroadcastHandler {
    fn handle_message(
        &mut self,
        packet: &Packet,
        state: &NodeInfo,
    ) -> Option<Vec<MessageResponse>> {
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
                match state.broadcast_topology.get(src) {
                    // Node is an internal Server Node
                    Some(nodes) => Some(
                        nodes
                            .iter()
                            .map(|n| MessageResponse::NoAck {
                                src: Option::Some(src.clone()),
                                dest: n.clone(),
                                in_reply_to: Option::None,
                                payload: Payload::Broadcast { message: *message },
                            })
                            .collect(),
                    ),
                    // Node is an external Client Node
                    None => {
                        let nodes = state.broadcast_topology[&state.node_id].iter();
                        let mut responses = Vec::with_capacity(nodes.len() + 1);

                        responses.push(MessageResponse::NoAck {
                            src: Option::None,
                            dest: src.clone(),
                            in_reply_to: *msg_id,
                            payload: Payload::BroadcastOk,
                        });

                        responses.extend(nodes.map(|n| MessageResponse::NoAck {
                            src: Option::Some(state.node_id.clone()),
                            dest: n.clone(),
                            in_reply_to: Option::None,
                            payload: Payload::Broadcast { message: *message },
                        }));

                        Some(responses)
                    }
                }
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
            } => Some(vec![MessageResponse::NoAck {
                src: Option::None,
                dest: src.clone(),
                in_reply_to: *msg_id,
                payload: Payload::ReadOk {
                    messages: self.messages.clone(),
                },
            }]),
            _ => None,
        }
    }
}
