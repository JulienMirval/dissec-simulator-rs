use std::cmp::Ordering;

use crate::common::Address;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum MessageType {
    Stop,
    RequestContribution,
    SendData,
    ScheduleHealthCheck,
    RequestHealth,
    ConfirmHealth,
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

#[derive(PartialEq, PartialOrd, Clone)]
pub struct Message {
    pub departure_time: f64,
    pub emitter: Address,
    pub arrival_time: Option<f64>,
    pub receiver: Address,
    pub message_type: MessageType,
}

impl Message {
    pub fn new(
        message_type: MessageType,
        departure_time: f64,
        emitter: Address,
        receiver: Address,
    ) -> Self {
        Message {
            departure_time,
            emitter,
            arrival_time: None,
            receiver,
            message_type,
        }
    }

    pub fn new_timeout(
        message_type: MessageType,
        emitter: Address,
        departure_time: f64,
        arrival_time: f64,
    ) -> Self {
        Message {
            departure_time,
            emitter,
            arrival_time: Some(arrival_time),
            receiver: emitter,
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
    use crate::common::Incrementable;

    use super::*;

    #[test]
    fn pings_are_handled_at_the_same_time() {
        let addr = Address::default();
        let a = Message::new(
            MessageType::RequestContribution,
            0.0,
            addr,
            addr.increment(None),
        );
        let b = Message::new(
            MessageType::RequestContribution,
            0.0,
            addr,
            addr.increment(None),
        );

        assert!(a.cmp(&b) == Ordering::Equal);
    }

    #[test]
    fn pings_are_handled_first() {
        let addr = Address::default();
        let a = Message::new(
            MessageType::RequestContribution,
            0.0,
            addr,
            addr.increment(None),
        );
        let b = Message::new(MessageType::SendData, 0.0, addr, addr.increment(None));

        assert!(a.cmp(&b) == Ordering::Greater);
    }
}
