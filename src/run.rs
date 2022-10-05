#[derive(Default, Clone)]
pub struct BuildingBlocks {
    pub full_failure_propagation: bool,
    pub local_failure_propagation: bool,
    pub aggregator_node_replacement: bool,
}

impl BuildingBlocks {
    pub fn default() -> BuildingBlocks {
        BuildingBlocks::minimal()
    }

    pub fn minimal() -> BuildingBlocks {
        BuildingBlocks {
            full_failure_propagation: true,
            local_failure_propagation: false,
            aggregator_node_replacement: false,
        }
    }
    pub fn tolerant() -> BuildingBlocks {
        BuildingBlocks {
            full_failure_propagation: false,
            local_failure_propagation: true,
            aggregator_node_replacement: false,
        }
    }
    pub fn resilient() -> BuildingBlocks {
        BuildingBlocks {
            full_failure_propagation: false,
            local_failure_propagation: true,
            aggregator_node_replacement: true,
        }
    }
}

#[derive(Default, Clone)]
pub struct CostsSettings {
    pub crypto: usize,
    pub comm: usize,
    pub compute: usize,
}

#[derive(Default, Clone)]
pub struct TreeSettings {
    pub fanout: u8,
    pub depth: u8,
    pub group_size: u8,
}

#[derive(Default, Clone)]
pub struct RunSettings {
    pub building_blocks: BuildingBlocks,
    pub average_failure_time: f64,
    pub costs: CostsSettings,
    pub tree: TreeSettings,
    pub seed: String,
}

impl RunSettings {
    pub fn tree_construction_latency(&self) -> f64 {
        (self.tree.depth as f64) * (self.costs.crypto * 4 + self.costs.comm * 2) as f64
    }
}
