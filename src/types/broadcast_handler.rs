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
                match state.broadcast_topology.get(src) {
                    // Node is an internal Server Node
                    Some(nodes) => {
                        eprintln!(
                            "Node {} forwarding Broadcast {} to {:?}",
                            state.node_id, message, nodes
                        );
                        for server in nodes {
                            send(MessageResponse {
                                src: Option::Some(src.clone()),
                                dest: server.clone(),
                                in_reply_to: Option::None,
                                payload: Payload::Broadcast { message: *message },
                            });
                        }
                    }
                    // Node is an external Client Node
                    None => {
                        send(MessageResponse {
                            src: Option::None,
                            dest: src.clone(),
                            in_reply_to: *msg_id,
                            payload: Payload::BroadcastOk,
                        });
                        let nodes = state.broadcast_topology[&state.node_id].iter();
                        eprintln!(
                            "Node {} sending first Broadcast {} to {:?}",
                            state.node_id, message, nodes
                        );
                        for server in nodes {
                            send(MessageResponse {
                                src: Option::Some(state.node_id.clone()),
                                dest: server.clone(),
                                in_reply_to: Option::None,
                                payload: Payload::Broadcast { message: *message },
                            });
                        }
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
            } => {
                send(MessageResponse {
                    src: Option::None,
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
