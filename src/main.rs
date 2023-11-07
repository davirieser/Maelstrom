#![allow(non_snake_case)]

use std::io::{stdin, stdout};

pub mod packet_handler;
pub use packet_handler::PacketHandler;

pub mod types;
pub use types::{
    collection::Collection,
    message::Message,
    message_handler::MessageHandler,
    message_response::MessageResponse,
    node_info::{NodeConnectionInfo, NodeInfo},
    packet::Packet,
    payload::Payload,
    topology::{BroadcastTopology, Topology},
};

pub mod handlers;
pub use handlers::{
    broadcast_handler::BroadcastHandler, echo_handler::EchoHandler,
    generate_handler::GenerateHandler,
};

fn main() {
    let stdin = stdin();
    let stdin_lock = stdin.lock();

    let stdout = stdout();
    let stdout_lock = stdout.lock();

    let mut handler = PacketHandler::new(stdin_lock, stdout_lock);

    let mut echo_handler = EchoHandler {};
    let mut generate_handler = GenerateHandler { counter: 0 };
    let mut broadcast_handler = BroadcastHandler { messages: vec![] };

    handler.add_handler(&mut echo_handler);
    handler.add_handler(&mut generate_handler);
    handler.add_handler(&mut broadcast_handler);

    handler.run();
}
