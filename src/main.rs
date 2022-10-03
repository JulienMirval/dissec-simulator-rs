mod common;
mod manager;
mod message;
mod node;
mod run;
mod tree_node;

use manager::Manager;
use run::TreeSettings;

fn main() {
    let mut manager = Manager::new(
        "42".to_string(),
        TreeSettings {
            fanout: 4,
            depth: 3,
            group_size: 3,
        },
    );

    manager.generate_tree_nodes();

    manager
        .nodes
        .get(&manager.querier_address)
        .unwrap()
        .tree_node
        .print(&manager, Some(0));
}
