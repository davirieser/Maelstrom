#![allow(non_snake_case)]

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

#[cfg(test)]
mod test {
    use crate::types::helpers::{build_broadcast_topology, is_lower_node_id};
    use crate::{BroadcastTopology, Topology};
    use std::cmp::Ordering;
    use std::collections::{HashMap, HashSet};

    fn cmp_node_ids(id1: &str, id2: &str) -> Ordering {
        (id1[1..].parse::<usize>().unwrap_or(0)).cmp(&id2[1..].parse::<usize>().unwrap_or(0))
    }

    #[test]
    fn test_is_lower_node_id() {
        assert!(is_lower_node_id("n1", "n2"));
        assert!(!is_lower_node_id("n1", "n1"));
        assert!(!is_lower_node_id("n2", "n1"));
    }

    #[test]
    fn test_build_broadcast_topology1() {
        let topology: Topology = HashMap::from([]);

        assert!(test_build_broadcast_topology_internal(topology));
    }
    #[test]
    fn test_build_broadcast_topology2() {
        let topology: Topology = HashMap::from([(String::from("n0"), vec![])]);

        assert!(test_build_broadcast_topology_internal(topology));
    }
    #[test]
    fn test_build_broadcast_topology3() {
        let topology: Topology =
            HashMap::from([(String::from("n0"), vec![]), (String::from("n1"), vec![])]);

        assert!(!test_build_broadcast_topology_internal(topology));
    }
    #[test]
    fn test_build_broadcast_topology4() {
        let topology: Topology = HashMap::from([
            (String::from("n0"), vec![String::from("n1")]),
            (String::from("n1"), vec![String::from("n0")]),
        ]);

        assert!(test_build_broadcast_topology_internal(topology));
    }
    #[test]
    fn test_build_broadcast_topology5() {
        let topology: Topology = HashMap::from([
            (
                String::from("n0"),
                vec![String::from("n1"), String::from("n2")],
            ),
            (
                String::from("n1"),
                vec![String::from("n3"), String::from("n0")],
            ),
            (
                String::from("n2"),
                vec![String::from("n3"), String::from("n0")],
            ),
            (
                String::from("n3"),
                vec![String::from("n1"), String::from("n2")],
            ),
        ]);

        assert!(test_build_broadcast_topology_internal(topology));
    }
    #[test]
    fn test_build_broadcast_topology6() {
        let topology: Topology = HashMap::from([
            (
                String::from("n0"),
                vec![String::from("n1"), String::from("n3")],
            ),
            (
                String::from("n1"),
                vec![String::from("n0"), String::from("n2"), String::from("n4")],
            ),
            (
                String::from("n2"),
                vec![String::from("n1"), String::from("n5")],
            ),
            (
                String::from("n3"),
                vec![String::from("n0"), String::from("n4")],
            ),
            (
                String::from("n4"),
                vec![String::from("n1"), String::from("n3"), String::from("n5")],
            ),
            (
                String::from("n5"),
                vec![String::from("n2"), String::from("n4")],
            ),
        ]);

        assert!(test_build_broadcast_topology_internal(topology));
        assert!(false);
    }

    fn test_build_broadcast_topology_internal(mut topology: Topology) -> bool {
        let all_topologies = build_broadcast_topology_internal(topology);
        println!("All Topologies: {:#?}", all_topologies);

        let complete_topology = collect_topologies(all_topologies);
        println!("Complete Topology: {:#?}", complete_topology);

        check_topology_is_complete(complete_topology)
    }
    fn check_topology_is_complete(mut topology: BroadcastTopology) -> bool {
        let mut valid = true;
        let mut nodes: HashSet<String> = topology.keys().cloned().collect();

        for node in nodes.iter() {
            // TODO: Check if any nodes are in the topology multiple times => Redundant Broadcast
            let adj = topology.get_mut(node).unwrap().iter().cloned().collect();

            let mut diff = nodes.difference(&adj).collect::<Vec<&String>>();
            let pos = diff.iter().position(|&i| i == node);
            if !pos.is_some() {
                println!("Self referencial Broadcast for Node {:?}", node);
                valid &= false;
            } else {
                diff.remove(pos.unwrap());
            }
            if !diff.is_empty() {
                println!(
                    "Nodes {:?} are missing for {:?}",
                    diff.into_iter().collect::<Vec<&String>>(),
                    node
                );
                valid &= false;
            }
        }

        valid
    }
    fn build_broadcast_topology_internal(
        base_topology: Topology,
    ) -> HashMap<String, BroadcastTopology> {
        let mut nodes: Vec<String> = base_topology.keys().cloned().collect();

        let mut complete_topology = HashMap::new();
        for node in nodes.iter() {
            let mut broadcast_topology = build_broadcast_topology(node, &nodes, &base_topology);

            complete_topology.insert(node.clone(), broadcast_topology);
        }

        complete_topology
    }
    fn collect_topologies(topologies: HashMap<String, BroadcastTopology>) -> Topology {
        let mut nodes: Vec<String> = topologies.keys().cloned().collect();
        let mut complete_topology: Topology = HashMap::with_capacity(nodes.len());

        for node in nodes.iter() {
            let topology = &topologies[node];
            for node in nodes.iter() {
                complete_topology
                    .entry(node.to_owned())
                    .or_default()
                    .extend(topology[node].iter().cloned());
            }
        }

        for node in nodes.iter() {
            let mut vec = complete_topology.get_mut(node).unwrap();
        }

        complete_topology
    }
}
