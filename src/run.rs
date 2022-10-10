use std::fmt;

#[derive(Default, Clone, Debug, PartialEq)]
pub enum FailureHandlingMode {
    #[default]
    FullFailurePropagation,
    LocalFailurePropagation,
    NodeReplacement,
}

impl fmt::Display for FailureHandlingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Default, Clone, Debug)]
pub struct BuildingBlocks {
    pub failure_handling: FailureHandlingMode,
}

impl BuildingBlocks {
    pub fn default() -> BuildingBlocks {
        BuildingBlocks::minimal()
    }

    pub fn minimal() -> BuildingBlocks {
        BuildingBlocks {
            failure_handling: FailureHandlingMode::FullFailurePropagation,
        }
    }
    pub fn tolerant() -> BuildingBlocks {
        BuildingBlocks {
            failure_handling: FailureHandlingMode::LocalFailurePropagation,
        }
    }
    pub fn resilient() -> BuildingBlocks {
        BuildingBlocks {
            failure_handling: FailureHandlingMode::NodeReplacement,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct CostsSettings {
    pub crypto: f64,
    pub comm: f64,
    pub compute: f64,
}

#[derive(Default, Clone, Debug)]
pub struct TreeSettings {
    pub fanout: u8,
    pub depth: u8,
    pub group_size: u8,
}

#[derive(Default, Clone, Debug)]
pub struct RunSettings {
    pub building_blocks: BuildingBlocks,
    pub average_failure_time: f64,
    pub health_check_period: f64,
    pub costs: CostsSettings,
    pub tree: TreeSettings,
    pub seed: String,
}

impl RunSettings {
    pub fn tree_construction_latency(&self) -> f64 {
        (self.tree.depth as f64) * self.costs.crypto * 4.0 + self.costs.comm * 2.0
    }
}
