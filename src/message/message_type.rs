use std::fmt;

#[derive(Copy, PartialEq, PartialOrd, Debug, Default)]
pub enum MessageType {
    #[default]
    Stop,
    RequestData,
    PrepareData,
    SendData,
    ScheduleHealthCheck,
    RequestHealth,
    ConfirmHealth,
    OpenChannel,
    ConfirmChannel,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Clone for MessageType {
    fn clone(&self) -> Self {
        match self {
            &default => default,
        }
    }
}

impl MessageType {
    pub fn priority(self) -> u8 {
        match self {
            MessageType::Stop => 255,
            MessageType::OpenChannel => 255,
            MessageType::ConfirmChannel => 255,
            MessageType::RequestData => 1,
            _ => 0,
        }
    }
}
