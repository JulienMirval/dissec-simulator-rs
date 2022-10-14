use std::{cmp::Ordering, fmt::Display};

use crate::common::Address;

use super::{MessageContent, MessageType};

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Message {
    pub delivered: bool,
    pub departure_time: f64,
    pub emitter: Address,
    pub arrival_time: f64,
    pub receiver: Address,
    pub message_type: MessageType,
    pub work: f64,
    pub content: MessageContent,
}

impl Message {
    pub fn new(
        message_type: MessageType,
        departure_time: f64,
        emitter: Address,
        arrival_time: f64,
        receiver: Address,
    ) -> Self {
        Message {
            delivered: false,
            departure_time,
            emitter,
            arrival_time,
            receiver,
            message_type,
            work: 0.0,
            content: MessageContent::default(),
        }
    }

    pub fn new_timeout(
        message_type: MessageType,
        emitter: Address,
        departure_time: f64,
        arrival_time: f64,
    ) -> Self {
        Message {
            delivered: false,
            departure_time,
            emitter,
            arrival_time,
            receiver: emitter,
            message_type,
            work: 0.0,
            content: MessageContent::default(),
        }
    }
}

impl Eq for Message {}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        let time_cmp = self.arrival_time.total_cmp(&other.arrival_time);
        if time_cmp == Ordering::Equal {
            if self.message_type.priority() > other.message_type.priority() {
                Ordering::Greater
            } else if self.message_type.priority() == other.message_type.priority() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else {
            time_cmp.reverse()
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "Message ({:?}) Node #{} @ {} -> Node #{} @ {}",
                self.message_type,
                self.emitter,
                self.departure_time,
                self.receiver,
                self.arrival_time
            )
            .as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Incrementable;

    use super::*;

    #[test]
    fn pings_are_handled_at_the_same_time() {
        let addr = Address::default();
        let a = Message::new(
            MessageType::RequestData,
            0.0,
            addr,
            0.0,
            addr.increment(None),
        );
        let b = Message::new(
            MessageType::RequestData,
            0.0,
            addr,
            0.0,
            addr.increment(None),
        );

        assert!(a.cmp(&b) == Ordering::Equal);
    }

    #[test]
    fn pings_are_handled_first() {
        let addr = Address::default();
        let a = Message::new(
            MessageType::RequestData,
            0.0,
            addr,
            0.0,
            addr.increment(None),
        );
        let b = Message::new(MessageType::SendData, 0.0, addr, 0.0, addr.increment(None));

        assert!(a.cmp(&b) == Ordering::Greater);
    }
}
