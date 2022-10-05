use crate::{common::Address, run::RunSettings, tree_node::TreeNode};

use super::{Node, NodeData, NodeRole};

pub struct ContributorNode {
    data: NodeData,
}

impl Node for ContributorNode {
    fn new(settings: RunSettings, address: Address) -> Box<ContributorNode> {
        let data = NodeData {
            settings,
            address,
            role: NodeRole::Contributor,
            local_time: 0.0,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
        };

        Box::new(ContributorNode { data })
    }

    fn settings(&self) -> &RunSettings {
        &self.settings()
    }
    fn data(&self) -> &NodeData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }
}
