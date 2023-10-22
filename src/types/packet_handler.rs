#![allow(unused)]

use serde_json::de::{IoRead, StreamDeserializer};
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::types::{
    helpers::build_broadcast_topology,
    message::Message,
    message_handler::MessageHandler,
    message_response::MessageResponse,
    node_info::{NodeConnectionInfo, NodeInfo},
    packet::Packet,
    payload::Payload,
};

pub struct PacketHandler<'de, 'a, I, O>
where
    I: Read,
    O: Write,
{
    stdin: StreamDeserializer<'de, IoRead<I>, Packet>,
    stdout: O,
    state: Option<NodeInfo>,
    handlers: Vec<&'a mut dyn MessageHandler>,
}

impl<'de, 'a, I, O> PacketHandler<'de, 'a, I, O>
where
    I: Read,
    O: Write,
{
    pub fn new(stdin: I, stdout: O) -> Self {
        let reader = serde_json::Deserializer::from_reader(stdin);
        let stdin = reader.into_iter();

        Self {
            stdin,
            stdout,
            state: Option::None,
            handlers: vec![],
        }
    }
    fn init(&mut self) {
        if let Some(Ok(Packet {
            src,
            body:
                Message {
                    msg_id,
                    payload:
                        Payload::Init {
                            node_id, node_ids, ..
                        },
                    ..
                },
            ..
        })) = self.stdin.next()
        {
            let node_number = node_id[1..].parse::<usize>().unwrap();
            let client_nodes = node_ids
                .iter()
                .filter(|id| id.starts_with('c'))
                .cloned()
                .collect();
            let server_nodes: Vec<String> = node_ids
                .iter()
                .filter(|id| id.starts_with('n') && **id != node_id)
                .cloned()
                .collect();

            let mut conn_info = HashMap::new();
            for node in server_nodes.iter() {
                conn_info.insert(
                    node.clone(),
                    NodeConnectionInfo {
                        out_msg_id: 0,
                        in_msg_id: 0,
                        un_ack_messages: Vec::new(),
                    },
                );
            }

            self.state = Option::Some(NodeInfo {
                node_id,
                node_ids,
                msg_number: 0,
                node_number,
                client_nodes,
                server_nodes,
                topology: Default::default(),
                broadcast_topology: Default::default(),
                conn_info,
            });
            self.send(Option::None, src, msg_id, Payload::InitOk);
        } else {
            panic!("Did not receive Init Message!");
        }
    }
    pub fn add_handler(&mut self, handler: &'a mut dyn MessageHandler) {
        self.handlers.push(handler);
    }
    pub fn step(&mut self) {
        if self.state.is_some() {
            match self.stdin.next() {
                Some(Ok(packet)) => {
                    let packets = self.handle_packet(packet);
                    // TODO: Send Packets
                    // TODO: Check if not acknowledged packets should be sent again.
                }
                Some(Err(e)) => eprintln!("Error parsing Message {}", e),
                None => panic!("No more Messages!"),
            }
        } else {
            self.init();
        }
    }
    fn handle_packet(&mut self, packet: Packet) -> Vec<Packet> {
        match packet {
            Packet {
                src,
                body:
                    Message {
                        msg_id,
                        payload: Payload::Init { .. },
                        ..
                    },
                ..
            } => {
                vec![Packet {
                    src: self.get_node_id().clone(),
                    dest: src,
                    body: Message {
                        msg_id: None,
                        in_reply_to: msg_id,
                        payload: Payload::Error {
                            code: 14,
                            text: "Got second Init Message".to_string(),
                        },
                    },
                }]
            }
            Packet {
                src,
                body:
                    Message {
                        msg_id,
                        payload: Payload::Topology { topology },
                        ..
                    },
                ..
            } => {
                let state = self.get_state_mut();
                state.broadcast_topology = build_broadcast_topology(state, &topology);
                state.topology = topology;

                vec![Packet {
                    src: self.get_node_id().clone(),
                    dest: src,
                    body: Message {
                        msg_id: None,
                        in_reply_to: None,
                        payload: Payload::TopologyOk,
                    },
                }]
            }
            Packet {
                body:
                    Message {
                        payload: Payload::Forward { packet },
                        ..
                    },
                ..
            } => {
                vec![*packet]
            }
            Packet {
                body:
                    Message {
                        payload: Payload::Batch { packets },
                        ..
                    },
                ..
            } => {
                // TODO: All Packets have same src and dest, change from Packets to Messages?
                let mut responses = Vec::new();
                for packet in packets {
                    responses.extend(self.handle_packet(packet));
                }
                responses 
            }
            Packet {
                src,
                body:
                    Message {
                        payload: Payload::Ack { messages },
                        in_reply_to,
                        ..
                    },
                ..
            } => {
                let state = self.get_state_mut();
                for message in messages {
                    self.ack_packet(&src, message);
                }
                Vec::with_capacity(0)
            }
            packet => {
                let mut handlers = std::mem::take(&mut self.handlers);
                let mut packets: Vec<Packet> = Vec::new();

                for handler in handlers.iter_mut() {
                    let responses = handler.handle_message(&packet, self.get_state());
                    // TODO: Extend Packets with returned MessageResponses.
                    if let Some(responses) = responses {
                        for response in responses {
                            match response {
                                MessageResponse::Ack {
                                    src,
                                    dest,
                                    in_reply_to,
                                    payload,
                                } => self.send_with_ack(src, dest, in_reply_to, payload),
                                MessageResponse::NoAck {
                                    src,
                                    dest,
                                    in_reply_to,
                                    payload,
                                } => self.send(src, dest, in_reply_to, payload),
                                MessageResponse::Response { payload } => {
                                    self.send(Option::None, packet.src.clone(), packet.body.msg_id, payload)
                                }
                                MessageResponse::ResponseWithAck { payload } => self.send_with_ack(
                                    Option::None,
                                    packet.src.clone(),
                                    packet.body.msg_id,
                                    payload,
                                ),
                            }
                        }
                    }
                }

                let _ = std::mem::replace(&mut self.handlers, handlers);

                packets
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
    fn get_node_id(&self) -> &String {
        &self.get_state().node_id
    }
    pub fn create_ack(&self, dest: String, msg_id: usize) -> Packet {
        Packet {
            src: self.get_state().node_id.clone(),
            dest,
            body: Message {
                msg_id: None,
                in_reply_to: None,
                payload: Payload::Ack {
                    messages: vec![msg_id],
                },
            },
        }
    }
    pub fn send_with_ack(
        &mut self,
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        payload: Payload,
    ) {
        self.send_inner(src, dest, in_reply_to, true, payload);
    }
    pub fn send(
        &mut self,
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        payload: Payload,
    ) {
        self.send_inner(src, dest, in_reply_to, false, payload);
    }
    fn send_inner(
        &mut self,
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        with_ack: bool,
        payload: Payload,
    ) {
        let src = src.unwrap_or_else(|| self.get_state().node_id.clone());
        let mut response = Packet {
            dest,
            src,
            body: Message {
                in_reply_to,
                msg_id: None,
                payload,
            },
        };

        if with_ack {
            if let Some(NodeConnectionInfo { out_msg_id, .. }) =
                self.get_state_mut().conn_info.get_mut(&response.src)
            {
                *out_msg_id += 1;
                response.body.msg_id = Option::Some(*out_msg_id);
                self.write_packet_with_ack(response);
            } else {
                self.write_packet(&response);
            }
        } else {
            self.write_packet(&response);
        }
    }
    fn write_packet_with_ack(&mut self, packet: Packet) -> bool {
        if packet.body.msg_id.is_some() {
            let packet_ref = match self.state.as_mut().unwrap().conn_info.get_mut(&packet.src) {
                Some(conn_info) => {
                    conn_info.un_ack_messages.push(packet);
                    conn_info.un_ack_messages.last().unwrap()
                }
                None => return false,
            };

            Self::write_packet_inner(self.stdout.by_ref(), packet_ref);

            true
        } else {
            false
        }
    }
    fn write_packet(&mut self, packet: &Packet) {
        Self::write_packet_inner(self.stdout.by_ref(), packet);
    }
    fn write_packet_inner(stdout: &mut O, packet: &Packet)
    where
        O: Write,
    {
        let _ = serde_json::to_writer(stdout.by_ref(), &packet);
        let _ = stdout.write(&[b'\n']);
        let _ = stdout.flush();
    }
    fn ack_packet(&mut self, src: &String, msg_id: usize) -> bool {
        match self.get_state_mut().conn_info.get_mut(src) {
            Some(conn_info) => {
                // NOTE: Use .position and .swap_remove because the ack's will probably come in the
                // same order as the packet are added to the vec. This means that binary search
                // would have to search until the start of the list (nearly) every time.
                // Cases in O-Notation:
                //      .position + .swap_remove = O(n) + O(1) = O(n)
                //      .binary_search + .remove = O(log(n)) + O(n) = O(n)
                match conn_info
                    .un_ack_messages
                    .iter()
                    .position(|p| p.body.msg_id.map_or(false, |id| id == msg_id))
                {
                    Some(idx) => {
                        conn_info.un_ack_messages.swap_remove(idx);
                        true
                    }
                    None => false,
                }
            }
            None => false,
        }
    }
    fn get_state(&self) -> &NodeInfo {
        self.state
            .as_ref()
            .expect("Tried responding before Init Message")
    }
    fn get_state_mut(&mut self) -> &mut NodeInfo {
        self.state
            .as_mut()
            .expect("Tried responding before Init Message")
    }
}
