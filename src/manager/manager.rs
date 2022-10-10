use crypto::{digest::Digest, sha2::Sha256};
use itertools::Itertools;
use rand::prelude::*;
use sorted_insert::SortedInsertBinary;
use std::collections::HashMap;

use crate::common::*;
use crate::message::Message;
use crate::node::{Node, QuerierNode};
use crate::run::{BuildingBlocks, CostsSettings, RunSettings, TreeSettings};

use super::Recording;

pub struct Manager {
    pub settings: RunSettings,
    pub nodes: HashMap<Address, Box<dyn Node>>,
    pub querier_address: Address,
    pub message_queue: Vec<Message>,
    pub current_time: f64,
    pub rng: SmallRng,
    pub recording: Recording,
}

impl Manager {
    pub fn default() -> Manager {
        Self::new(
            BuildingBlocks::default(),
            "str".to_string(),
            TreeSettings {
                fanout: 4,
                depth: 3,
                group_size: 3,
            },
        )
    }
    pub fn new(building_blocks: BuildingBlocks, seed: String, tree: TreeSettings) -> Manager {
        let mut hasher = Sha256::new();
        hasher.input_str(seed.as_ref());
        let mut seed_bytes: [u8; 32] = [0; 32];
        hasher.result(&mut seed_bytes);

        let manager = Manager {
            settings: RunSettings {
                building_blocks: building_blocks.clone(),
                average_failure_time: 10000.0,
                health_check_period: 1000.0,
                costs: CostsSettings {
                    crypto: 100.0,
                    comm: 100.0,
                    compute: 0.0,
                },
                tree,
                seed,
            },
            querier_address: 0_usize,
            nodes: HashMap::new(),
            message_queue: vec![],
            current_time: 0.0,
            rng: SmallRng::from_seed(seed_bytes),
            recording: Recording::new(building_blocks, true),
        };

        manager
    }

    /// Creates all the nodes in the tree and initializes them
    pub fn setup(&mut self) {
        // Create the querier group
        let mut querier_group: Box<dyn Node> =
            QuerierNode::new(self.settings.clone(), self.querier_address);
        querier_group.data_mut().tree_node.members = (0..self.settings.tree.group_size)
            .map(|_| self.querier_address)
            .collect();
        self.nodes.insert(self.querier_address, querier_group);

        // Create the tree below the querier
        Manager::create_tree_node(
            self,
            self.querier_address,
            self.settings.tree.depth,
            self.settings.tree.depth,
            self.querier_address.increment(None),
        );

        self.current_time = self.settings.tree_construction_latency();

        self.generate_failures();

        // Initialize the tree with the failures that occured during tree creation
        // self.initialize_tree_failures();

        self.initialize_nodes();
    }

    pub fn handle_next_message(&mut self) -> bool {
        let msg = self.message_queue.pop();

        if let Some(mut msg) = msg {
            let resulting_messages = self
                .nodes
                .get_mut(&msg.receiver)
                .unwrap()
                .handle_message(&mut msg);

            if resulting_messages.is_none() {
                // Message bounced, queue it back
                self.insert_message(msg);
            } else {
                self.current_time = msg.arrival_time;
                resulting_messages
                    .unwrap()
                    .iter()
                    .sorted()
                    .for_each(|resulting_message| self.insert_message(resulting_message.clone()));
                self.recording.record(&msg);
            }
            true
        } else {
            false
        }
    }

    pub fn insert_message(&mut self, msg: Message) {
        self.message_queue.sorted_insert_asc_binary(msg);
    }
}

#[cfg(test)]
mod tests {
    use crate::{message::MessageType, run::TreeSettings};

    use super::*;

    #[test]
    fn handle_message() {
        let mut manager = Manager::new(
            BuildingBlocks::default(),
            "str".to_string(),
            TreeSettings {
                fanout: 4,
                depth: 3,
                group_size: 3,
            },
        );
        manager.setup();
        manager.message_queue.clear();

        let arrival_time = 1000.0;
        let emitter: Address = 0;
        let receiver: Address = 0;
        manager.insert_message(Message::new(
            MessageType::ScheduleHealthCheck,
            0.0,
            emitter,
            arrival_time,
            receiver,
        ));

        manager.handle_next_message();

        assert_eq!(manager.current_time, arrival_time);
        assert_eq!(
            manager.nodes.get(&receiver).unwrap().data().local_time,
            arrival_time
        );
    }

    #[test]
    fn test_message_insertion() {
        let mut manager = Manager::new(
            BuildingBlocks::default(),
            "str".to_string(),
            TreeSettings {
                fanout: 4,
                depth: 3,
                group_size: 3,
            },
        );
        manager.setup();
        manager.message_queue.clear();

        let emitter: Address = 0;
        let receiver: Address = 0;
        let iterations = 10;
        let step_size = 100.0;

        for i in 0..iterations {
            manager.insert_message(Message::new(
                MessageType::ConfirmHealth,
                0.0,
                emitter,
                (i as f64) * step_size,
                receiver,
            ));
        }

        for i in 0..iterations {
            assert_eq!(
                manager.message_queue.last().unwrap().arrival_time,
                (i as f64) * step_size
            );
            manager.handle_next_message();
        }
    }
}
