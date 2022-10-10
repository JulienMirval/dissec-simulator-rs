#![feature(total_cmp)] // Needed to compare floats

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

    manager.setup();

    manager
        .nodes
        .get(&manager.querier_address)
        .unwrap()
        .data()
        .tree_node
        .print(&manager, Some(0));

    let mut done = false;
    while !done {
        manager.handle_next_message();

        let msg = manager.message_queue.last();
        if msg.is_none() {
            done = true;
        }
    }
}
