#![allow(non_snake_case)]

mod types;

#[cfg(test)]
mod test {
    use crate::types::{
        helpers::{build_broadcast_topology, is_lower_node_id},
        packet_handler::NodeInfo,
    };
    use std::cmp::Ordering;
    use std::collections::HashMap;

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
        let topology: HashMap<String, Vec<String>> = HashMap::from([
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
        let calculated: HashMap<String, Vec<String>> = HashMap::from([
            (
                String::from("n0"),
                vec![String::from("n1"), String::from("n2")],
            ),
            (String::from("n1"), vec![String::from("n2")]),
            (String::from("n2"), vec![String::from("n1")]),
            (String::from("n3"), vec![]),
        ]);
        // TODO: Need to calculate for "n1", "n2" and "n3"

        let node_ids: Vec<String> = ["n0", "n1", "n2", "n3"]
            .into_iter()
            .map(|s| String::from(s))
            .collect();
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
            messages: Default::default(),
            msg_ids: Default::default(),
        };
        let mut broadcast_topology = build_broadcast_topology(&mut state, &topology);

        for vec in broadcast_topology.values_mut() {
            vec.sort_by(|s1, s2| cmp_node_ids(s1.as_str(), s2.as_str()));
        }

        assert_eq!(calculated, broadcast_topology);
    }
}
