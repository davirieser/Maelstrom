#![allow(unused)]

use debug_print::{debug_eprint, debug_eprintln};
use serde_json::de::{IoRead, StreamDeserializer};
use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{Read, Write},
};

use crate::types::{
    collection::Collection,
    helpers::build_broadcast_topology,
    message::Message,
    message_handler::MessageHandler,
    message_response::MessageResponse,
    node_info::{MessageSyncStatus, NodeConnectionInfo, NodeInfo},
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
            debug_eprintln!("Got Init Message");

            let node_number = node_id[1..].parse::<usize>().unwrap();
            let client_nodes = node_ids
                .iter()
                .filter(|id| id.starts_with('c'))
                .cloned()
                .collect();
            let server_nodes: Vec<String> = node_ids
                .iter()
                .filter(|id| id.starts_with('n'))
                .cloned()
                .collect();

            let mut conn_info = HashMap::new();
            for node in server_nodes.iter() {
                conn_info.insert(
                    node.clone(),
                    NodeConnectionInfo {
                        out_msg_id: 0,
                        in_msg_id: MessageSyncStatus::Synced { last_msg_id: 0 },
                        un_ack_messages: Default::default(),
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

            let ok_packet = Packet {
                src: self.state.as_ref().unwrap().node_id.clone(),
                dest: src,
                body: Message {
                    in_reply_to: msg_id,
                    msg_id: None,
                    payload: Payload::InitOk,
                },
            };
            self.write_packet(ok_packet);
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
                    debug_eprintln!("Got {:#?}", packet);

                    let mut send_sync_request = false;
                    if let Some(mut conn_info) = self.get_state_mut().conn_info.get_mut(&packet.src)
                    {
                        if let Some(msg_id) = packet.body.msg_id {
                            match conn_info.in_msg_id.is_next_msg_id(msg_id) {
                                // NOTE: If packet msg_id is lower than the expected one,
                                // this packet has already been received => immidiately return.
                                Ordering::Less => return,
                                Ordering::Equal => conn_info.in_msg_id.increment_msg_id(),
                                // NOTE: If packet msg_id is higher than the expected one,
                                // some packets have not been received => Add to missing msg_ids
                                // and send sync request to source node.
                                Ordering::Greater => {
                                    conn_info.in_msg_id.add_missing_msg_ids(msg_id);
                                    send_sync_request = true;
                                }
                            }
                        }

                        if let Some(in_reply_to) = packet.body.in_reply_to {
                            Self::ack_packet_inner(conn_info, in_reply_to);
                        }
                    }

                    let mut packets = match send_sync_request {
                        true => Collection::One(Packet {
                            src: packet.dest.clone(),
                            dest: packet.src.clone(),
                            body: Message {
                                msg_id: None,
                                in_reply_to: None,
                                payload: Payload::SyncRequest,
                            },
                        }),
                        false => Collection::None,
                    };
                    packets += self.handle_packet(packet);
                    match packets {
                        Collection::None => {}
                        Collection::One(packet) => {
                            self.write_packet(packet);
                        }
                        Collection::Multiple(packets) => {
                            let dict: HashMap<String, Vec<Message>> =
                                packets.into_iter().fold(HashMap::new(), |mut acc, packet| {
                                    acc.entry(packet.dest).or_default().push(packet.body);
                                    acc
                                });

                            for kvp in dict {
                                let (dest, messages) = kvp;
                                match messages.len() {
                                    0 => {}
                                    1 => self.write_packet(Packet {
                                        src: self.get_node_id().clone(),
                                        dest,
                                        body: messages.into_iter().next().unwrap(),
                                    }),
                                    _ => self.write_batch(dest, messages),
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => eprintln!("Error parsing Message {}", e),
                None => panic!("No more Messages!"),
            }
        } else {
            self.init();
        }
    }
    fn write_batch(&mut self, dest: String, messages: Vec<Message>) {
        let packet = Packet {
            src: self.get_node_id().clone(),
            dest: dest.clone(),
            body: Message {
                msg_id: None,
                in_reply_to: None,
                payload: Payload::Batch { messages },
            },
        };
        Self::write_packet_inner(self.stdout.by_ref(), &packet);

        match packet.body.payload {
            Payload::Batch { messages } => {
                for message in messages.into_iter().filter(|m| m.msg_id.is_some()) {
                    self.add_packet_to_ack(Packet {
                        src: self.get_node_id().clone(),
                        dest: dest.clone(),
                        body: message,
                    });
                }
            }
            _ => panic!("How did this happen?"),
        }
    }
    fn create_error_packet(
        &self,
        in_reply_to: Option<usize>,
        dest: String,
        code: usize,
        text: String,
    ) -> Packet {
        Packet {
            src: self.get_node_id().clone(),
            dest,
            body: Message {
                msg_id: None,
                in_reply_to,
                payload: Payload::Error { code, text },
            },
        }
    }
    fn handle_packet(&mut self, packet: Packet) -> Collection<Packet> {
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
            } => Collection::One(self.create_error_packet(
                msg_id,
                src,
                14,
                "Got second Init Message".to_string(),
            )),
            // NOTE: Single Ack Packets are acked using their in_reply_to Field in the step
            // function.
            Packet {
                body:
                    Message {
                        payload: Payload::Ack,
                        ..
                    },
                ..
            } => Collection::None,
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
                // NOTE: Rebuilding the Topology every time a Topology Packet is sent is fine.
                let state = self.get_state_mut();
                state.broadcast_topology =
                    build_broadcast_topology(&state.node_id, &state.server_nodes, &topology);
                state.topology = topology;

                debug_eprintln!("Got Topology: {:#?}", state.topology);
                debug_eprintln!("Built Broadcast Topology: {:#?}", state.broadcast_topology);

                Collection::One(Packet {
                    src: self.get_node_id().clone(),
                    dest: src,
                    body: Message {
                        msg_id: None,
                        in_reply_to: msg_id,
                        payload: Payload::TopologyOk,
                    },
                })
            }
            Packet {
                body:
                    Message {
                        payload: Payload::Forward { packet },
                        ..
                    },
                ..
            } => Collection::One(*packet),
            Packet {
                src,
                body:
                    Message {
                        payload: Payload::SyncRequest,
                        ..
                    },
                ..
            } => {
                if let Some(messages) = self
                    .get_state_mut()
                    .conn_info
                    .get_mut(&src)
                    .map(|m| &mut m.un_ack_messages)
                {
                    let messages = std::mem::replace(messages, Vec::with_capacity(0));
                    messages
                        .into_iter()
                        .map(|m| Packet {
                            src: self.get_node_id().clone(),
                            dest: src.clone(),
                            body: m,
                        })
                        .collect::<Vec<Packet>>()
                        .into()
                } else {
                    Collection::None
                }
            }
            Packet {
                src,
                dest,
                body:
                    Message {
                        payload: Payload::Batch { messages },
                        ..
                    },
            } => {
                let mut responses = Vec::new();

                for message in messages {
                    let packet = Packet {
                        src: src.clone(),
                        dest: dest.clone(),
                        body: message,
                    };
                    match self.handle_packet(packet) {
                        Collection::None => {}
                        Collection::One(packet) => responses.push(packet),
                        Collection::Multiple(packets) => responses.extend(packets),
                    }
                }

                Collection::Multiple(responses)
            }
            Packet {
                src,
                body:
                    Message {
                        payload: Payload::MultiAck { messages },
                        in_reply_to,
                        ..
                    },
                ..
            } => {
                let state = self.get_state_mut();

                for message in messages {
                    self.ack_packet(&src, message);
                }

                Collection::None
            }
            packet => {
                let mut handlers = std::mem::take(&mut self.handlers);
                let mut packets: Vec<Packet> = Vec::new();

                for handler in handlers.iter_mut() {
                    let responses = handler.handle_message(&packet, self.get_state());
                    match responses {
                        Collection::None => {}
                        Collection::One(response) => {
                            packets.push(self.create_packet(&packet, response))
                        }
                        Collection::Multiple(responses) => packets.extend(
                            responses
                                .into_iter()
                                .map(|r| self.create_packet(&packet, r)),
                        ),
                    };
                }

                let _ = std::mem::replace(&mut self.handlers, handlers);

                packets.into()
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }
    fn create_packet(&mut self, trigger: &Packet, response: MessageResponse) -> Packet {
        match response {
            MessageResponse::Ack {
                src,
                dest,
                in_reply_to,
                payload,
            } => {
                let src = src.unwrap_or_else(|| self.get_state().node_id.clone());
                let msg_id = self.get_state_mut().conn_info.get_mut(&src).map(|o| {
                    o.out_msg_id += 1;
                    o.out_msg_id
                });
                Packet {
                    src,
                    dest,
                    body: Message {
                        msg_id,
                        in_reply_to,
                        payload,
                    },
                }
            }
            MessageResponse::NoAck {
                src,
                dest,
                in_reply_to,
                payload,
            } => {
                let src = src.unwrap_or_else(|| self.get_state().node_id.clone());
                Packet {
                    src,
                    dest,
                    body: Message {
                        msg_id: None,
                        in_reply_to,
                        payload,
                    },
                }
            }
            MessageResponse::Response { payload } => {
                let src = trigger.dest.clone();
                let dest = trigger.src.clone();
                Packet {
                    src,
                    dest,
                    body: Message {
                        in_reply_to: trigger.body.msg_id,
                        msg_id: None,
                        payload,
                    },
                }
            }
            MessageResponse::ResponseWithAck { payload } => {
                let src = trigger.dest.clone();
                let dest = trigger.src.clone();
                let msg_id = self.get_state_mut().conn_info.get_mut(&src).map(|o| {
                    let id = o.out_msg_id;
                    o.out_msg_id += 1;
                    id
                });
                Packet {
                    src,
                    dest,
                    body: Message {
                        in_reply_to: trigger.body.msg_id,
                        msg_id,
                        payload,
                    },
                }
            }
        }
    }
    pub fn get_node_id(&self) -> &String {
        &self.get_state().node_id
    }
    pub fn create_ack(&self, dest: String, msg_id: usize) -> Packet {
        Packet {
            src: self.get_state().node_id.clone(),
            dest,
            body: Message {
                msg_id: None,
                in_reply_to: Some(msg_id),
                payload: Payload::Ack,
            },
        }
    }
    pub fn write_packet(&mut self, packet: Packet) {
        Self::write_packet_inner(self.stdout.by_ref(), &packet);

        self.add_packet_to_ack(packet);
    }
    fn add_packet_to_ack(&mut self, packet: Packet) {
        if packet.body.msg_id.is_some() {
            if let Some(conn_info) = self.state.as_mut().unwrap().conn_info.get_mut(&packet.src) {
                conn_info.un_ack_messages.push(packet.body);
            }
        }
    }
    fn write_packet_inner(stdout: &mut O, packet: &Packet)
    where
        O: Write,
    {
        debug_eprintln!("Send {:#?}", packet);

        let _ = serde_json::to_writer(stdout.by_ref(), &packet);
        let _ = stdout.write(&[b'\n']);
        let _ = stdout.flush();
    }
    fn ack_packet(&mut self, src: &String, msg_id: usize) -> bool {
        match self.get_state_mut().conn_info.get_mut(src) {
            Some(conn_info) => Self::ack_packet_inner(conn_info, msg_id),
            None => false,
        }
    }
    fn ack_packet_inner(conn_info: &mut NodeConnectionInfo, msg_id: usize) -> bool {
        // NOTE: Use .position and .swap_remove because the ack's will probably come in the
        // same order as the packet are added to the vec. This means that binary search
        // would have to search until the start of the list (nearly) every time.
        // Cases in O-Notation:
        //      .position + .swap_remove = O(n) + O(1) = O(n)
        //      .binary_search + .remove = O(log(n)) + O(n) = O(n)
        match conn_info
            .un_ack_messages
            .iter()
            .position(|m| m.msg_id.map_or(false, |id| id == msg_id))
        {
            Some(idx) => {
                conn_info.un_ack_messages.swap_remove(idx);
                true
            }
            None => false,
        }
    }
    pub fn get_state(&self) -> &NodeInfo {
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
