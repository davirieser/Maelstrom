use serde_json::de::{IoRead, StreamDeserializer};
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::types::{
    helpers::build_broadcast_topology, message::Message, message_handler::MessageHandler,
    packet::Packet, payload::Payload,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum MessageSyncState {
    Synced(usize),
    MissingMessages(Vec<usize>),
}

#[derive(Debug)]
pub struct NodeInfo {
    pub node_id: String,
    pub client_nodes: Vec<String>,
    pub server_nodes: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub broadcast_topology: HashMap<String, Vec<String>>,
    pub node_number: usize,
    pub node_ids: Vec<String>,
    pub msg_number: usize,
    pub msg_ids: HashMap<String, MessageSyncState>,
    pub messages: HashMap<String, Vec<String>>,
}

pub struct MessageResponse {
    pub src: Option<String>,
    pub dest: String,
    pub in_reply_to: Option<usize>,
    pub payload: Payload,
}

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

            let mut messages = HashMap::with_capacity(node_ids.len());
            for node in server_nodes.iter() {
                messages.insert(node.clone(), vec![]);
            }
            let mut msg_ids = HashMap::with_capacity(node_ids.len());
            for node in server_nodes.iter() {
                msg_ids.insert(node.clone(), MessageSyncState::Synced(0));
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
                messages,
                msg_ids,
            });
            self.respond(Option::None, src, msg_id, Payload::InitOk);
        } else {
            panic!("Did not receive Init Message!");
        }
    }
    pub fn add_handler(&mut self, handler: &'a mut dyn MessageHandler) {
        self.handlers.push(handler);
    }
    pub fn handle_message(&mut self) {
        if self.state.is_some() {
            match self.stdin.next() {
                Some(Ok(packet)) => self.handle_packet(packet),
                Some(Err(e)) => eprintln!("Error parsing Message {}", e),
                None => panic!("No more Messages!"),
            }
        } else {
            self.init();
        }
    }
    fn handle_packet(&mut self, packet: Packet) {
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
                self.respond(
                    Option::None,
                    src,
                    msg_id,
                    Payload::Error {
                        code: 14,
                        text: "Got second Init Message".to_string(),
                    },
                );
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
                self.respond(Option::None, src, msg_id, Payload::TopologyOk {});
                let state = self
                    .state
                    .as_mut()
                    .expect("State not initialised during Topology");
                state.broadcast_topology = build_broadcast_topology(state, &topology);
                state.topology = topology;
            }
            Packet {
                body:
                    Message {
                        payload: Payload::Forward { packet },
                        ..
                    },
                ..
            } => {
                self.write_packet(&packet);
            }
            packet => {
                let mut handlers = std::mem::take(&mut self.handlers);

                for handler in handlers.iter_mut() {
                    let mut responses = vec![];
                    let mut closure = |r| responses.push(r);
                    handler.handle_message(
                        &packet,
                        self.state.as_ref().expect("State was not initialised!"),
                        &mut closure,
                    );
                    for response in responses {
                        // TODO: Store Messages until Ack is sent.
                        self.respond(
                            response.src,
                            response.dest,
                            response.in_reply_to,
                            response.payload,
                        );
                    }
                }

                let _ = std::mem::replace(&mut self.handlers, handlers);
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            self.handle_message();
        }
    }
    pub fn respond(
        &mut self,
        src: Option<String>,
        dest: String,
        in_reply_to: Option<usize>,
        payload: Payload,
    ) {
        let state = self
            .state
            .as_mut()
            .expect("Tried responding before Init Message");

        let src = src.unwrap_or_else(|| state.node_id.clone());
        let response = Packet {
            dest,
            src,
            body: Message {
                in_reply_to,
                msg_id: Some(state.msg_number),
                payload,
            },
        };

        state.msg_number += 1;

        self.write_packet(&response);
    }
    fn write_packet(&mut self, packet: &Packet) {
        let _ = serde_json::to_writer(self.stdout.by_ref(), &packet);
        let _ = self.stdout.write("\n".as_bytes());
        let _ = self.stdout.flush();
    }
}
