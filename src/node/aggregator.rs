use crate::common::Address;

use super::{BaseNode, Node, NodeData, NodeRole};

pub struct AggregatorNode {
    node: BaseNode,
}

impl Node for AggregatorNode {
    fn new(address: Address) -> Box<AggregatorNode> {
        let mut node = BaseNode::new(address);
        node.data_mut().role = NodeRole::Aggregator;

        Box::new(AggregatorNode { node: *node })
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
