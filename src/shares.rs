use std::{
    collections::hash_map::DefaultHasher,
    fmt,
    hash::{Hash, Hasher},
};

use crate::common::Address;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Share {
    pub share: f64,
    pub count: usize,
    pub id: String,
}

impl Share {
    pub fn new(value: f64, sender: Address) -> Share {
        Share {
            share: value,
            count: 1,
            id: sender.to_string(),
        }
    }
}

impl fmt::Display for Share {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait AggregatableShares {
    fn aggregate(self: &Self) -> Share;
}

impl AggregatableShares for Vec<Share> {
    fn aggregate(&self) -> Share {
        let mut hasher = DefaultHasher::new();
        self.iter().for_each(|share| share.id.hash(&mut hasher));
        Share {
            share: self.iter().map(|share| share.share).sum(),
            count: self.iter().map(|share| share.count).sum(),
            id: hasher.finish().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shares::Share;

    use super::AggregatableShares;

    #[test]
    fn create_tree() {
        let a = Share::new(1.0, 123);
        let b = Share::new(2.0, 125);
        let c = Share::new(3.0, 1243);
        let v = vec![a, b, c];
        let result = v.aggregate();

        assert_eq!(result.share, 6.0);
        assert_eq!(result.count, 3);
        assert_eq!(result.id, "14127686999214930996");
    }
}
