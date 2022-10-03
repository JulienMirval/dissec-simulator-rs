use crypto::{digest::Digest, sha2::Sha256};
use itertools::Itertools;
use rand::prelude::*;
use rand_distr::Exp;
use std::collections::{BinaryHeap, HashMap};

use crate::common::*;
use crate::message::Message;
use crate::node::{AggregatorNode, ContributorNode, LeafAggregatorNode, Node, QuerierNode};
use crate::run::{BuildingBlocks, CostsSettings, RunSettings, TreeSettings};

pub struct Manager {
    pub settings: RunSettings,
    pub seed: String,
    pub nodes: HashMap<Address, Box<dyn Node>>,
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
                costs: CostsSettings {
                    crypto: 100,
                    comm: 100,
                    compute: 0,
                },
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
                .data()
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
                    .data_mut()
                    .tree_node
                    .children
                    .push(group_nodes.clone());
            }

            let mut next_node = group_nodes.last().unwrap().increment(Some(1));
            for addr in group_nodes.clone() {
                let mut node: Box<dyn Node> = if current_depth > 1 {
                    AggregatorNode::new(addr)
                } else if current_depth == 1 {
                    LeafAggregatorNode::new(addr)
                } else {
                    ContributorNode::new(addr)
                };

                node.data_mut().tree_node.depth = current_depth;
                node.data_mut().tree_node.members = group_nodes.clone();
                node.data_mut().tree_node.parents = manager
                    .nodes
                    .get(&parent_group_first_node)
                    .unwrap()
                    .data()
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
        let mut querier_group: Box<dyn Node> = QuerierNode::new(self.querier_address);
        querier_group.data_mut().tree_node.members = (0..self.settings.tree.group_size)
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
                .filter(|(_, x)| x.data().death_time < self.current_time)
                .count();

            // Parent signs, backup answers, parent confirms, backup verifies.
            // Then backup signs and asks members, they verify and answer with children
            let replacement_time: f64 =
                (10 * self.settings.costs.crypto + 8 * self.settings.costs.comm) as f64;
            self.current_time += replacement_time * (failed_nodes as f64);
        } else if self.settings.building_blocks.local_failure_propagation {
            // Failed nodes are dropped by their parents
            let failed_nodes: Vec<_> = self
                .nodes
                .iter()
                .filter(|(_, x)| x.data().death_time < self.current_time)
                .map(|(_, x)| x.data().address)
                .collect();

            // Stop subtrees recursively
            // Note: stopping too many nodes, tree creation should continue despite failures when not on the leader
            let mut stopped_nodes: usize = 0;
            fn stop_child(manager: &mut Manager, stopped_nodes: &mut usize, target: &Address) {
                let node = manager.nodes.get(target).unwrap().data().tree_node.clone();
                *stopped_nodes += node.children.len();

                for member in &node.members {
                    manager.nodes.get_mut(member).unwrap().data_mut().death_time = 0.0;
                }
                for child in node.children {
                    stop_child(manager, stopped_nodes, child.first().unwrap());
                }
            }
            for node in &failed_nodes {
                stop_child(self, &mut stopped_nodes, node);
            }
        } else if self.settings.building_blocks.full_failure_propagation {
            // self.current_time = self
            //     .nodes
            //     .iter()
            //     .map(|(_, x)| x.data().death_time)
            //     .reduce(|a, b| if a > b { b } else { a })
            //     .unwrap();
        }
    }

    /// Sets the time of death of each node in the simulation.
    fn generate_failures(&mut self) {
        let exp = Exp::new(1.0 / self.settings.average_failure_time).unwrap();

        for (_, node) in &mut self.nodes {
            node.data_mut().death_time = exp.sample(&mut self.rng);
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
            .for_each(|(_, x)| assert_ne!(x.data().death_time, 0.0));

        manager.generate_failures();
    }
}
