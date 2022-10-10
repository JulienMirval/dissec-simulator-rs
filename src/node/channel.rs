use crate::common::Address;

#[derive(Debug)]
pub struct ChannelState {
    pub peer_address: Address,
    pub maintained: bool,
}

impl ChannelState {
    pub fn new(peer: Address, maintained: bool) -> ChannelState {
        ChannelState {
            peer_address: peer,
            maintained,
        }
    }
}
