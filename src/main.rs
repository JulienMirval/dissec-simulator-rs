#![feature(total_cmp)] // Needed to compare floats
#![feature(derive_default_enum)]

mod common;
mod manager;
mod message;
mod node;
mod run;
mod shares;
mod tree_node;

use manager::Manager;

fn main() {
    let mut manager = Manager::default();

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

    manager.recording.final_contributors = manager
        .nodes
        .iter()
        .filter(|(_, node)| node.data().death_time > manager.current_time)
        .count();

    if let Err(err) = manager.recording.write_to_path(
        format!("{}.csv", chrono::offset::Utc::now())
            .replace(":", "_")
            .replace(" ", ".")
            .as_str(),
    ) {
        println!("Failed writing records: {}", err);
    }
}
