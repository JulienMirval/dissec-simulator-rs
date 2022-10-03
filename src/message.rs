use std::cmp::{Ordering, Reverse};

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum MessageType {
    Stop,
    RequestContribution,
    SendData,
}

impl MessageType {
    fn priority(self) -> u8 {
        match self {
            MessageType::Stop => 255,
            MessageType::RequestContribution => 1,
            _ => 0,
        }
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Message {
    pub arrival_time: u64,
    pub message_type: MessageType,
}

impl Message {
    pub fn new(message_type: MessageType) -> Self {
        Message {
            arrival_time: 0,
            message_type,
        }
    }
}

impl Eq for Message {}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.arrival_time < other.arrival_time {
            Ordering::Greater
        } else {
            if self.message_type.priority() > other.message_type.priority() {
                Ordering::Greater
            } else if self.message_type.priority() == other.message_type.priority() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pings_are_handled_at_the_same_time() {
        let a = Message::new(MessageType::RequestContribution);
        let b = Message::new(MessageType::RequestContribution);

        assert!(a.cmp(&b) == Ordering::Equal);
    }

    #[test]
    fn pings_are_handled_first() {
        let a = Message::new(MessageType::RequestContribution);
        let b = Message::new(MessageType::SendData);

        assert!(a.cmp(&b) == Ordering::Greater);
    }
}
