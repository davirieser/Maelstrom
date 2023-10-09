#![allow(unused, non_snake_case)]

use std::io::{stdin, stdout};

mod types;
use types::{
    broadcast_handler::BroadcastHandler, echo_handler::EchoHandler,
    generate_handler::GenerateHandler, packet_handler::PacketHandler,
    topology_handler::TopologyHandler,
};

fn main() {
    let stdin = stdin();
    let stdin_lock = stdin.lock();

    let stdout = stdout();
    let stdout_lock = stdout.lock();

    let mut handler = PacketHandler::new(stdin_lock, stdout_lock);

    let mut echo_handler = EchoHandler {};
    // let mut generate_handler = GenerateHandler {};
    let mut broadcast_handler = BroadcastHandler { messages: vec![] };
    let mut topology_handler = TopologyHandler {};

    handler.add_handler(&mut echo_handler);
    // handler.add_handler(&mut generate_handler);
    handler.add_handler(&mut broadcast_handler);

    handler.add_handler(&mut topology_handler);

    handler.run();
}
