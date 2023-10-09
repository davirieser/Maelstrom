
use serde_json::de::{IoRead, StreamDeserializer};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::cmp::Ordering;

use crate::types::{
    message::Message, message_handler::MessageHandler, packet::Packet, payload::Payload,
};

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
}

pub struct MessageResponse {
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
            let server_nodes = node_ids
                .iter()
                .filter(|id| id.starts_with('n') && **id != node_id)
                .cloned()
                .collect();

            self.state = Option::Some(NodeInfo {
                node_id,
                node_ids,
                msg_number: 0,
                node_number,
                client_nodes,
                server_nodes,
                topology: Default::default(),
                broadcast_topology: Default::default(),
            });
            self.respond(src, msg_id, Payload::InitOk);
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
                Some(Ok(Packet {
                    src,
                    body:
                        Message {
                            msg_id,
                            payload: Payload::Init { .. },
                            ..
                        },
                    ..
                })) => {
                    self.respond(
                        src,
                        msg_id,
                        Payload::Error {
                            code: 14,
                            text: "Got second Init Message".to_string(),
                        },
                    );
                }
                Some(Ok(Packet {
                    src,
                    body:
                        Message {
                            msg_id,
                            payload: Payload::Topology { topology },
                            ..
                        },
                    ..
                })) => {
                    eprintln!("Topology: {:?}", topology);
                    self.respond(src, msg_id, Payload::TopologyOk {});
                    let state = self
                        .state
                        .as_mut()
                        .expect("State not initialised during Topology");
                    state.broadcast_topology = build_broadcast_topology(state, &topology);
                    state.topology = topology;
                }
                Some(Ok(packet)) => {
                    let mut handlers = std::mem::take(&mut self.handlers);

                    for handler in handlers.iter_mut() {
                        let mut responses = vec![];
                        let mut closure = |r| responses.push(r);
                        handler.handle_message(&packet, self.state.as_ref().unwrap(), &mut closure);
                        for response in responses {
                            self.respond(response.dest, response.in_reply_to, response.payload);
                        }
                    }

                    let _ = std::mem::replace(&mut self.handlers, handlers);
                }
                Some(Err(e)) => eprintln!("Error parsing Message {}", e),
                None => panic!("No more Messages!"),
            }
        } else {
            self.init();
        }
    }
    pub fn run(&mut self) {
        loop {
            self.handle_message();
        }
    }
    pub fn respond(&mut self, dest: String, in_reply_to: Option<usize>, payload: Payload) {
        let state = self
            .state
            .as_mut()
            .expect("Tried responding before Init Message");
        let src = std::mem::take(&mut state.node_id);

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

        let _ = self
            .stdout
            .write(serde_json::to_string(&response).unwrap().as_bytes());
        let _ = self.stdout.write("\n".as_bytes());
        let _ = self.stdout.flush();

        let _ = std::mem::replace(&mut state.node_id, response.src);
    }
}

fn build_broadcast_topology(
    state: &mut NodeInfo,
    topology: &HashMap<String, Vec<String>>,
) -> HashMap<String, Vec<String>> {
    let own_node_id = &state.node_id;
    let num_nodes = state.server_nodes.len();

    let mut broadcast_topology: HashMap<String, Vec<String>> = HashMap::new();

    for node_id in &state.node_ids {
        let mut already_visited: HashSet<String> = HashSet::new();

        let mut neighbours: HashSet<String> = topology[own_node_id].clone().into_iter().collect();
        neighbours.remove(node_id);

        if node_id == own_node_id {
            broadcast_topology.insert(node_id.clone(), neighbours.into_iter().collect());
            continue;
        }

        eprintln!("Node: {}", node_id);

        let mut stack: HashSet<String> = HashSet::from([node_id.clone()]);
        eprintln!("{} < {} && {} && {}", already_visited.len(), num_nodes, !stack.is_empty(), !neighbours.is_empty());
        while (already_visited.len() < num_nodes && !stack.is_empty() && !neighbours.is_empty()) {
            eprintln!("\tNeighbours: {:?}, Stack: {:?}, Already visited: {:?}", neighbours, stack, already_visited);
            let mut temp: HashSet<String> = HashSet::new();

            if stack.contains(own_node_id) {
                temp = stack
                    .iter()
                    .filter(|n| is_lower_node_id(n, own_node_id))
                    .flat_map(|n| topology[n].clone())
                    .collect();
            } else {
                temp = stack.iter().flat_map(|n| topology[n].clone()).collect();
            }

            temp = temp.difference(&already_visited).cloned().collect();
            neighbours = neighbours.difference(&temp).cloned().collect();
            already_visited.extend(stack);
            stack = temp;
        }

        eprintln!("Inserting {:?} for {}", neighbours, node_id);
        broadcast_topology.insert(node_id.clone(), neighbours.into_iter().collect());
    }

    eprintln!("Broadcast Topology: {:?}", broadcast_topology);

    broadcast_topology
}

fn is_lower_node_id(id1: &str, id2: &str) -> bool {
    id1[1..].parse::<usize>().unwrap_or(0) < id2[1..].parse::<usize>().unwrap_or(0)
}

fn cmp_node_ids(id1: &str, id2: &str) -> Ordering {
    (id1[1..].parse::<usize>().unwrap_or(0)).cmp(&id2[1..].parse::<usize>().unwrap_or(0))
}

mod test {
    use super::*;

    #[test]
    fn test_is_lower_node_id() {
        assert!(is_lower_node_id("n1", "n2"));
        assert!(!is_lower_node_id("n1", "n1"));
        assert!(!is_lower_node_id("n2", "n1"));
    }

    #[test]
    fn test_build_broadcast_topology1() {
        let topology : HashMap<String, Vec<String>> = HashMap::from([
            (String::from("n0"), vec![String::from("n1"), String::from("n2")]),
            (String::from("n1"), vec![String::from("n3"), String::from("n0")]),
            (String::from("n2"), vec![String::from("n3"), String::from("n0")]),
            (String::from("n3"), vec![String::from("n1"), String::from("n2")]),
        ]);
        let calculated : HashMap<String, Vec<String>> = HashMap::from([
            (String::from("n0"), vec![String::from("n1"), String::from("n2")]),
            (String::from("n1"), vec![String::from("n2")]),
            (String::from("n2"), vec![String::from("n1")]),
            (String::from("n3"), vec![]),
        ]);
        // TODO: Need to calculate for "n1", "n2" and "n3"

        let node_ids : Vec<String> = ["n0", "n1", "n2", "n3"].into_iter().map(|s| String::from(s)).collect();
        let server_nodes = node_ids.clone();

        let mut state = NodeInfo {
            msg_number: 0,
            node_number: 4,
            node_id: String::from("n0"),
            node_ids, 
            server_nodes,
            client_nodes: vec![], 
            topology: HashMap::new(),
            broadcast_topology: HashMap::new(),
        };
        let mut broadcast_topology = build_broadcast_topology(&mut state, &topology);

        for vec in broadcast_topology.values_mut() {
            vec.sort_by(|s1, s2| cmp_node_ids(s1.as_str(), s2.as_str()));
        }

        assert_eq!(calculated, broadcast_topology);
    }
}
