use crate::common::Address;

use super::{BaseNode, Node, NodeData, NodeRole};

pub struct ContributorNode {
    node: BaseNode,
}

impl Node for ContributorNode {
    fn new(address: Address) -> Box<ContributorNode> {
        let mut node = BaseNode::new(address);
        node.data_mut().role = NodeRole::Contributor;

        Box::new(ContributorNode { node: *node })
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
