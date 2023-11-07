use crate::types::topology::Topology;
use std::collections::{HashMap, HashSet};

pub(crate) fn build_broadcast_topology(
    own_node_id: &String,
    all_nodes: &Vec<String>,
    topology: &Topology,
) -> HashMap<String, Vec<String>> {
    let num_nodes = all_nodes.len();

    let mut broadcast_topology: HashMap<String, Vec<String>> = HashMap::new();

    for node_id in all_nodes {
        let mut already_visited: HashSet<String> = HashSet::with_capacity(num_nodes);

        let mut neighbours: HashSet<String> = topology[own_node_id].clone().into_iter().collect();
        neighbours.remove(node_id);

        if node_id == own_node_id {
            broadcast_topology.insert(node_id.clone(), neighbours.into_iter().collect());
            continue;
        }

        let mut found_self = false;
        let mut stack: HashSet<String> = HashSet::from([node_id.clone()]);
        // TODO: If The Stack is empty condition is met, then there probably is two or more non-connected sub-graphs
        while !found_self
            && already_visited.len() < num_nodes
            && !stack.is_empty()
            && !neighbours.is_empty()
        {
            let mut temp: HashSet<String>;

            if stack.contains(own_node_id) {
                temp = stack
                    .iter()
                    .filter(|n| is_lower_node_id(n, own_node_id))
                    .flat_map(|n| topology[n].clone())
                    .collect();
                found_self = true;
            } else {
                temp = stack.iter().flat_map(|n| topology[n].clone()).collect();
            }

            temp = temp.difference(&already_visited).cloned().collect();
            neighbours = neighbours.difference(&temp).cloned().collect();
            already_visited.extend(stack);
            stack = temp;
        }

        broadcast_topology.insert(node_id.clone(), neighbours.into_iter().collect());
    }

    broadcast_topology
}

pub(crate) fn is_lower_node_id(id1: &str, id2: &str) -> bool {
    id1[1..].parse::<usize>().unwrap_or(0) < id2[1..].parse::<usize>().unwrap_or(0)
}
