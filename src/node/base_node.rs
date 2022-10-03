use crate::{common::Address, tree_node::TreeNode};

use super::{Node, NodeData, NodeRole};

pub struct BaseNode {
    data: NodeData,
}

impl Node for BaseNode {
    fn new(address: Address) -> Box<BaseNode> {
        Box::new(BaseNode {
            data: NodeData {
                address,
                role: NodeRole::Replacement,
                death_time: 0.0,
                opened_channels: vec![],
                tree_node: TreeNode::new(address),
            },
        })
    }

    fn data(&self) -> &NodeData {
        &self.data
    }

    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    fn initialize(&mut self) {}

    fn handle_send_data(&mut self) {}
}
