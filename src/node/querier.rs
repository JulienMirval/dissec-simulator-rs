use std::collections::HashMap;

use crate::{common::Address, message::Message, run::RunSettings, tree_node::TreeNode};

use super::{Node, NodeData, NodeRole};

pub struct QuerierNode {
    data: NodeData,
}

impl Node for QuerierNode {
    fn new(settings: RunSettings, address: Address) -> Box<QuerierNode> {
        let data = NodeData {
            settings,
            address,
            role: NodeRole::Querier,
            local_time: 0.0,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
            finished_working: false,
            aggregates: HashMap::new(),
            secret_value: 50.0,
        };

        Box::new(QuerierNode { data })
    }

    fn data(&self) -> &NodeData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    fn handle_send_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} is receiving data from node #{}",
            msg.receiver, msg.emitter
        );
        let mut resulting_messages = vec![];

        let aggregate = msg.content.data.clone().unwrap();
        self.data_mut().aggregates.insert(msg.emitter, aggregate);

        let expected_data = self
            .data()
            .tree_node
            .children
            .iter()
            .map(|child_group| {
                // HACK: This handler should be implemented for each role
                if self.data().role == NodeRole::LeafAggregator {
                    // The child is a contributor
                    self.data().aggregates.get(&child_group[0])
                } else {
                    let position = self
                        .data()
                        .tree_node
                        .members
                        .iter()
                        .position(|&member| self.data().address == member)
                        .unwrap();
                    self.data().aggregates.get(&child_group[position])
                }
            })
            .collect::<Vec<_>>();
        let received_all_data = expected_data.iter().all(|data| data.is_some());
        if received_all_data {
            println!("Finished! Propagating stop...");
        }

        resulting_messages
    }
}
