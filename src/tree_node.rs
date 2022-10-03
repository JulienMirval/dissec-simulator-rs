use crate::{common::Address, manager::Manager};

pub struct TreeNode {
    pub address: Address,
    pub members: Vec<Address>,
    pub parents: Vec<Address>,
    pub children: Vec<Vec<Address>>,
}

impl TreeNode {
    pub fn new(address: Address) -> TreeNode {
        TreeNode {
            address,
            members: vec![],
            parents: vec![],
            children: vec![],
        }
    }

    pub fn print(&self, manager: &Manager, depth: Option<u8>) {
        let depth = depth.unwrap_or(0);
        let tabs = (0..depth).map(|_| "\t").collect::<Vec<_>>().join("");
        println!(
            "{}Group {}",
            tabs,
            self.members
                .iter()
                .map(|x| format!("#{} ({})", x, manager.nodes.get(&x).unwrap().role))
                .collect::<Vec<_>>()
                .join(", ")
        );

        for child in &self.children {
            manager
                .nodes
                .get(child.first().unwrap())
                .unwrap()
                .tree_node
                .print(manager, Some(depth + 1));
        }
    }
}
