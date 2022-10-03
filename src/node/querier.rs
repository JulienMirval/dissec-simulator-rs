use crate::common::Address;

use super::{BaseNode, Node, NodeData, NodeRole};

pub struct QuerierNode {
    node: BaseNode,
}

impl Node for QuerierNode {
    fn new(address: Address) -> Box<QuerierNode> {
        let mut node = BaseNode::new(address);
        node.data_mut().role = NodeRole::Aggregator;

        Box::new(QuerierNode { node: *node })
    }

    fn data(&self) -> &NodeData {
        &self.node.data()
    }

    fn data_mut(&mut self) -> &mut NodeData {
        self.node.data_mut()
    }

    fn initialize(&mut self) {}

    fn handle_send_data(&mut self) {}
}
