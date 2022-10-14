use std::fmt;

use crate::{common::Address, shares::Share};

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct MessageContent {
    pub data: Option<Share>,
    pub target_node: Option<Address>,
}

impl fmt::Display for MessageContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
