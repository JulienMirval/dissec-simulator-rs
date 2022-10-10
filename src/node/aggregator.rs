use crate::{common::Address, run::RunSettings, tree_node::TreeNode};

use super::{Node, NodeData, NodeRole};

pub struct AggregatorNode {
    data: NodeData,
}

impl Node for AggregatorNode {
    fn new(settings: RunSettings, address: Address) -> Box<AggregatorNode> {
        let mut data = NodeData {
            settings,
            address,
            role: NodeRole::Replacement,
            local_time: 0.0,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
        };
        data.role = NodeRole::Aggregator;

        Box::new(AggregatorNode { data })
    }

    fn settings(&self) -> &RunSettings {
        &self.data().settings
    }
    fn data(&self) -> &NodeData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }
}
