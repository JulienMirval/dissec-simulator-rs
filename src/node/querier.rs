use crate::{common::Address, run::RunSettings, tree_node::TreeNode};

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
        };

        Box::new(QuerierNode { data })
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
