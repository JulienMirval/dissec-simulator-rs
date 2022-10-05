use std::cmp::Ordering;

use itertools::Itertools;
use rand::Rng;
use rand_distr::{Distribution, Exp};

use crate::{
    common::*,
    message::{Message, MessageType},
    node::*,
};

use super::Manager;

impl Manager {
    /// Sets the time of death of each node in the simulation.
    pub(super) fn generate_failures(&mut self) {
        let exp = Exp::new(1.0 / self.settings.average_failure_time).unwrap();

        for (_, node) in &mut self.nodes {
            node.data_mut().death_time = exp.sample(&mut self.rng);
        }
    }

    /// Initializes the channels between nodes and send initial messages
    pub(super) fn initialize_nodes(&mut self) {
        for (&node_address, node) in self.nodes.iter_mut() {
            let position = node
                .data()
                .tree_node
                .members
                .iter()
                .position(|x| x == &node.data().tree_node.address)
                .unwrap();

            // Contributors don't monitor health
            if node.data().role != NodeRole::Contributor {
                self.message_heap.push(Message::new(
                    MessageType::ScheduleHealthCheck,
                    self.current_time,
                    node_address,
                    node_address,
                ));
            }

            if node.data().role == NodeRole::Querier {
                // Channels with children
                for child in node.data().tree_node.children[0].clone() {
                    node.data_mut()
                        .opened_channels
                        .push(ChannelState::new(child, true));
                }
            } else if node.data().role == NodeRole::Aggregator {
                // Channels with parent
                let parent_address = node.data().tree_node.parents[position];
                node.data_mut()
                    .opened_channels
                    .push(ChannelState::new(parent_address, false));

                // Leader opens with members
                if position == 0 {
                    let members = node
                        .data()
                        .tree_node
                        .members
                        .clone()
                        .into_iter()
                        .filter(|x| x != &node.data().tree_node.address)
                        .collect::<Vec<_>>();

                    for member in members {
                        node.data_mut()
                            .opened_channels
                            .push(ChannelState::new(member, false));
                    }
                } else {
                    let leader = node.data().tree_node.members[0];
                    node.data_mut()
                        .opened_channels
                        .push(ChannelState::new(leader, false));
                }

                // Channels with children
                for group in node.data().tree_node.children.clone() {
                    println!(
                        "#{} role {} {:?} position {} child {:?}",
                        node.data().address,
                        node.data().role,
                        node.data().tree_node.children,
                        position,
                        group
                    );
                    node.data_mut()
                        .opened_channels
                        .push(ChannelState::new(group[position], true));
                }
            } else if node.data().role == NodeRole::LeafAggregator {
                // Channels with parent
                let parent_address = node.data().tree_node.parents[position];
                node.data_mut()
                    .opened_channels
                    .push(ChannelState::new(parent_address, false));

                // Leader opens with members
                let members = node
                    .data()
                    .tree_node
                    .members
                    .clone()
                    .into_iter()
                    .filter(|x| x != &node.data().tree_node.address)
                    .collect::<Vec<_>>();

                for member in members {
                    node.data_mut()
                        .opened_channels
                        .push(ChannelState::new(member, true));
                }
            }
        }
    }

    /// Initialize the tree to failures that occured during the tree creation
    pub(super) fn initialize_tree_failures(&mut self) {
        if self.settings.building_blocks.aggregator_node_replacement {
            // When replacing nodes, extend the construction by the time it takes to replace a node times the number of failures
            let replacement_time: f64 =
                (10 * self.settings.costs.crypto + 8 * self.settings.costs.comm) as f64;

            let nodes_sorted = self
                .nodes
                .iter()
                .map(|(_, x)| x)
                .sorted_by(|a, b| {
                    if a.data().death_time > b.data().death_time {
                        Ordering::Greater
                    } else if a.data().death_time == b.data().death_time {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                })
                .collect::<Vec<_>>();

            let mut i = 0;
            println!("{}", nodes_sorted.get(i).unwrap().data().death_time);
            while nodes_sorted.get(i).is_some()
                && self.current_time > nodes_sorted.get(i).unwrap().data().death_time
            {
                self.current_time += replacement_time;
                i += 1;
            }
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

    /// Recursive tree creation
    /// Only define neighbors, does not open channels or send messages
    pub(super) fn create_tree_node(
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
                AggregatorNode::new(manager.settings.clone(), addr)
            } else if current_depth == 1 {
                LeafAggregatorNode::new(manager.settings.clone(), addr)
            } else {
                ContributorNode::new(manager.settings.clone(), addr)
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
                next_node = Manager::create_tree_node(
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
                            ..(manager.settings.tree.fanout * manager.settings.tree.fanout) as f64,
                    )
                    .to_int_unchecked();

                for _ in 0..number_of_contributors {
                    next_node = Manager::create_tree_node(
                        manager,
                        starting_address,
                        max_depth,
                        0,
                        next_node,
                    );
                }
            }
        }

        next_node
    }
}

#[cfg(test)]
mod tests {
    use crate::run::TreeSettings;

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

        manager.setup();

        assert_eq!(manager.nodes.len(), 224);

        manager
            .nodes
            .iter()
            .for_each(|(_, x)| assert_ne!(x.data().death_time, 0.0));

        manager.generate_failures();
    }
}
