use crypto::{digest::Digest, sha2::Sha256};
use rand::prelude::*;
use std::collections::{BinaryHeap, HashMap};

use crate::common::*;
use crate::message::Message;
use crate::node::{Node, QuerierNode};
use crate::run::{BuildingBlocks, CostsSettings, RunSettings, TreeSettings};

pub struct Manager {
    pub settings: RunSettings,
    pub nodes: HashMap<Address, Box<dyn Node>>,
    pub querier_address: Address,
    pub message_heap: BinaryHeap<Message>,
    pub current_time: f64,
    pub rng: SmallRng,
}

impl Manager {
    pub fn new(seed: String, tree: TreeSettings) -> Manager {
        let mut hasher = Sha256::new();
        hasher.input_str(seed.as_ref());
        let mut seed_bytes: [u8; 32] = [0; 32];
        hasher.result(&mut seed_bytes);

        let manager = Manager {
            settings: RunSettings {
                building_blocks: BuildingBlocks::resilient(),
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
            message_heap: BinaryHeap::new(),
            current_time: 0.0,
            rng: SmallRng::from_seed(seed_bytes),
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
        let msg = self.message_heap.pop();

        if let Some(mut msg) = msg {
            self.nodes
                .get_mut(&msg.receiver)
                .unwrap()
                .handle_message(&mut msg)
                .iter()
                .for_each(|resulting_message| self.message_heap.push(resulting_message.clone()));

            true
        } else {
            false
        }
    }
}
