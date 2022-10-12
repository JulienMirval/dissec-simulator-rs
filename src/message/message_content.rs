use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct MessageContent {
    pub data: Option<f64>,
}

impl fmt::Display for MessageContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
