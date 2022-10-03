use crate::{common::Address, tree_node::TreeNode};

use super::NodeRole;

pub struct ChannelState {
    pub peer_address: Address,
}

pub struct NodeData {
    pub address: Address,
    pub role: NodeRole,
    pub death_time: f64,
    pub opened_channels: Vec<ChannelState>,
    pub tree_node: TreeNode,
}

pub trait Node {
    fn new(address: Address) -> Box<Self>
    where
        Self: Sized;

    fn data(&self) -> &NodeData;
    fn data_mut(&mut self) -> &mut NodeData;

    fn initialize(&mut self);

    fn handle_send_data(&mut self);
}
