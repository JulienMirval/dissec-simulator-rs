use crypto::{digest::Digest, sha2::Sha256};
use itertools::Itertools;
use rand::prelude::*;
use rand_distr::Exp;
use std::collections::{BinaryHeap, HashMap};

use crate::common::*;
use crate::message::Message;
use crate::node::{Node, NodeRole};
use crate::run::{BuildingBlocks, RunSettings, TreeSettings};

pub struct Manager {
    pub settings: RunSettings,
    pub seed: String,
    pub nodes: HashMap<Address, Node>,
    pub querier_address: Address,
    pub message_heap: BinaryHeap<Message>,
    pub current_time: f64,
    rng: SmallRng,
}

impl Manager {
    pub fn new(seed: String, tree: TreeSettings) -> Manager {
        let mut hasher = Sha256::new();
        hasher.input_str(seed.as_ref());
        let mut seed_bytes: [u8; 32] = [0; 32];
        hasher.result(&mut seed_bytes);

        let manager = Manager {
            settings: RunSettings {
                building_blocks: BuildingBlocks::tolerant(),
                average_failure_time: 10000.0,
                tree,
            },
            seed,
            querier_address: 0_usize,
            nodes: HashMap::new(),
            message_heap: BinaryHeap::new(),
            current_time: 0.0,
            rng: SmallRng::from_seed(seed_bytes),
        };

        return manager;
    }

    /// Creates all the nodes in the tree and initializes them
    pub fn generate_tree_nodes(&mut self) {
        // Recursive tree creation
        fn create_tree_node(
            manager: &mut Manager,
            parent_group_first_node: Address,
            max_depth: u8,
            current_depth: u8,
            starting_address: Address,
        ) -> Address {
            let group_nodes = if current_depth > 0 {
                // Aggregator group
                (0..manager.settings.tree.group_size)
                    .map(|i| starting_address.increment(Some(i as usize)))
                    .collect()
            } else {
                vec![starting_address]
            };

            // Update the parent's child
            for member in manager
                .nodes
                .get(&parent_group_first_node)
                .unwrap()
                .tree_node
                .members
                .clone()
                .iter()
                .unique()
            {
                manager
                    .nodes
                    .get_mut(&member)
                    .unwrap()
                    .tree_node
                    .children
                    .push(group_nodes.clone());
            }

            let mut next_node = group_nodes.last().unwrap().increment(Some(1));
            for addr in group_nodes.clone() {
                let mut node = Node::new(
                    addr,
                    if current_depth > 1 {
                        NodeRole::Aggregator
                    } else if current_depth == 1 {
                        NodeRole::LeafAggregator
                    } else {
                        NodeRole::Contributor
                    },
                );
                node.tree_node.members = group_nodes.clone();
                node.tree_node.parents = manager
                    .nodes
                    .get(&parent_group_first_node)
                    .unwrap()
                    .tree_node
                    .members
                    .clone();
                manager.nodes.insert(addr, node);
            }

            if current_depth > 1 {
                for _ in 0..manager.settings.tree.fanout {
                    next_node = create_tree_node(
                        manager,
                        starting_address,
                        max_depth,
                        current_depth - 1,
                        next_node,
                    );
                }
            } else if current_depth == 1 {
                // The node is a leaf aggregator
                unsafe {
                    let number_of_contributors = manager
                        .rng
                        .gen_range(
                            (manager.settings.tree.fanout as f64)
                                ..(manager.settings.tree.fanout * manager.settings.tree.fanout)
                                    as f64,
                        )
                        .to_int_unchecked();

                    for _ in 0..number_of_contributors {
                        next_node =
                            create_tree_node(manager, starting_address, max_depth, 0, next_node);
                    }
                }
            }

            next_node
        }

        // Create the querier group
        let mut querier_group = Node::new(self.querier_address, NodeRole::Querier);
        querier_group.tree_node.members = (0..self.settings.tree.group_size)
            .map(|_| self.querier_address)
            .collect();
        self.nodes.insert(self.querier_address, querier_group);

        // Create the tree below the querier
        create_tree_node(
            self,
            self.querier_address,
            self.settings.tree.depth,
            self.settings.tree.depth,
            self.querier_address.increment(None),
        );

        self.current_time = self.settings.building_blocks.tree_construction_latency();

        // Initialize the tree with the failures that occured during tree creation
        self.generate_failures();
        if self.settings.building_blocks.aggregator_node_replacement {
            // When replacing nodes, extend the construction by the time it takes to replace a node times the number of failures
            let failed_nodes = self
                .nodes
                .iter()
                .filter(|(_, x)| x.death_time < self.current_time)
                .count();

            let replacement_time: f64 = 0.0;
            self.current_time += replacement_time * (failed_nodes as f64);
        } else if self.settings.building_blocks.local_failure_propagation {
            // Failed nodes are detected by their parents
        }
    }

    /// Sets the time of death of each node in the simulation.
    fn generate_failures(&mut self) {
        let exp = Exp::new(1.0 / self.settings.average_failure_time).unwrap();

        for (_, node) in &mut self.nodes {
            node.death_time = exp.sample(&mut self.rng);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tree() {
        let mut manager = Manager::new(
            "str".to_string(),
            TreeSettings {
                fanout: 4,
                depth: 3,
                group_size: 3,
            },
        );

        manager.generate_tree_nodes();

        assert_eq!(manager.nodes.len(), 224);

        manager
            .nodes
            .iter()
            .for_each(|(_, x)| assert_ne!(x.death_time, 0.0));

        manager.generate_failures();
    }
}
