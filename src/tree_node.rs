use crate::{common::Address, manager::Manager};

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub address: Address,
    pub depth: u8,
    pub members: Vec<Address>,
    pub parents: Vec<Address>,
    pub children: Vec<Vec<Address>>,
}

impl TreeNode {
    pub fn new(address: Address) -> TreeNode {
        TreeNode {
            address,
            depth: 0,
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
                .map(|x| format!("#{} ({})", x, manager.nodes.get(&x).unwrap().data().role))
                .collect::<Vec<_>>()
                .join(", ")
        );

        for child in &self.children {
            manager
                .nodes
                .get(child.first().unwrap())
                .unwrap()
                .data()
                .tree_node
                .print(manager, Some(depth + 1));
        }
    }
}
