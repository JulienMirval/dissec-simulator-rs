use std::fmt::{Display, Formatter, Result};

use crate::{common::Address, tree_node::TreeNode};

pub struct ChannelState {
    pub peer_address: Address,
}

pub enum NodeRole {
    Querier,
    Aggregator,
    LeafAggregator,
    Contributor,
}

impl Display for NodeRole {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Self::Querier => formatter.write_str("Querier"),
            Self::Aggregator => formatter.write_str("Aggregator"),
            Self::LeafAggregator => formatter.write_str("LeafAggregator"),
            Self::Contributor => formatter.write_str("Contributor"),
        }
    }
}

pub struct Node {
    pub address: Address,
    pub role: NodeRole,
    pub death_time: f64,
    pub opened_channels: Vec<ChannelState>,
    pub tree_node: TreeNode,
}

impl Node {
    pub fn new(address: Address, role: NodeRole) -> Node {
        Node {
            address,
            role,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
        }
    }
}
