use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq)]
pub enum NodeRole {
    Querier,
    Aggregator,
    LeafAggregator,
    Contributor,
    Replacement,
}

impl Display for NodeRole {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Self::Querier => formatter.write_str("Querier"),
            Self::Aggregator => formatter.write_str("Aggregator"),
            Self::LeafAggregator => formatter.write_str("LeafAggregator"),
            Self::Contributor => formatter.write_str("Contributor"),
            Self::Replacement => formatter.write_str("Replacement"),
        }
    }
}
