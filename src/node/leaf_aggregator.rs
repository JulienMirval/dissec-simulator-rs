use std::collections::HashMap;

use crate::{
    common::Address,
    message::{Message, MessageType},
    run::RunSettings,
    tree_node::TreeNode,
};

use super::{Node, NodeData, NodeRole};

pub struct LeafAggregatorNode {
    data: NodeData,
}

impl Node for LeafAggregatorNode {
    fn new(settings: RunSettings, address: Address) -> Box<LeafAggregatorNode> {
        let data = NodeData {
            settings,
            address,
            role: NodeRole::LeafAggregator,
            local_time: 0.0,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
            finished_working: false,
            aggregates: HashMap::new(),
            secret_value: 50.0,
        };

        Box::new(LeafAggregatorNode { data })
    }

    fn data(&self) -> &NodeData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    fn setup(&mut self, current_time: f64) -> Vec<crate::message::Message> {
        let mut messages = vec![];
        messages.push(Message::new(
            MessageType::ScheduleHealthCheck,
            current_time,
            self.data().address,
            current_time,
            self.data().address,
        ));

        let is_leader = self
            .data
            .tree_node
            .members
            .iter()
            .position(|&member| self.data.address == member)
            .unwrap()
            == 0;
        if is_leader {
            for &child in self.data.tree_node.children.iter().flatten() {
                messages.push(Message::new(
                    MessageType::RequestData,
                    current_time,
                    self.data().address,
                    current_time + self.message_latency(),
                    child,
                ));
            }
        }

        messages
    }
}
