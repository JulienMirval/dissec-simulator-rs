use std::fmt;

#[derive(Copy, PartialEq, PartialOrd, Debug, Default)]
pub enum MessageType {
    #[default]
    Stop,
    RequestContribution,
    SendData,
    ScheduleHealthCheck,
    RequestHealth,
    ConfirmHealth,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Clone for MessageType {
    fn clone(&self) -> Self {
        match self {
            Self::Stop => Self::Stop,
            Self::RequestContribution => Self::RequestContribution,
            Self::SendData => Self::SendData,
            Self::ScheduleHealthCheck => Self::ScheduleHealthCheck,
            Self::RequestHealth => Self::RequestHealth,
            Self::ConfirmHealth => Self::ConfirmHealth,
        }
    }
}

impl MessageType {
    pub fn priority(self) -> u8 {
        match self {
            MessageType::Stop => 255,
            MessageType::RequestContribution => 1,
            _ => 0,
        }
    }
}
