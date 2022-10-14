use std::collections::HashMap;

use crate::{
    common::Address,
    message::{Message, MessageType},
    run::RunSettings,
    shares::Share,
    tree_node::TreeNode,
};

use super::{Node, NodeData, NodeRole};

pub struct ContributorNode {
    data: NodeData,
    shares: Vec<Share>,
}

impl Node for ContributorNode {
    fn new(settings: RunSettings, address: Address) -> Box<ContributorNode> {
        let data = NodeData {
            settings,
            address,
            role: NodeRole::Contributor,
            local_time: 0.0,
            death_time: 0.0,
            opened_channels: vec![],
            tree_node: TreeNode::new(address),
            finished_working: false,
            aggregates: HashMap::new(),
            secret_value: 50.0,
        };

        Box::new(ContributorNode {
            data,
            shares: vec![],
        })
    }

    fn data(&self) -> &NodeData {
        &self.data
    }
    fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    fn setup(&mut self, _current_time: f64) -> Vec<crate::message::Message> {
        let messages = vec![];

        messages
    }

    fn handle_request_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received a data request from node #{}",
            msg.receiver, msg.emitter
        );
        let mut resulting_messages = vec![];

        if self.shares.len() == 0 {
            // Prepare the shares
            let mut buffer = 0.0;
            for _ in 0..(self.data.settings.tree.group_size - 1) {
                let rng = 10000.0;
                buffer += rng;
                self.shares
                    .push(Share::new(self.data.secret_value + rng, self.data.address));
            }
            self.shares.push(Share::new(
                self.data.secret_value - buffer,
                self.data.address,
            ));

            // Verify the query
            self.data.local_time += 3.0 * self.data().settings.costs.crypto;

            // HACK: Contributors discover their parents when they receive the request.
            // Here, the knowledge is given by default
            for parent in &self.data.tree_node.parents {
                let mut msg = Message::new(
                    MessageType::PrepareData,
                    self.data.local_time,
                    self.data.address,
                    self.data.local_time + self.message_latency(),
                    self.data.address,
                );
                msg.content.target_node = Some(*parent);

                resulting_messages.push(msg);
            }
        } else {
            // Request coming from a replacement node
        }

        resulting_messages
    }
    fn handle_prepare_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} is preparing to send data to node #{:?}",
            msg.receiver, msg.content.target_node
        );
        let mut resulting_messages = vec![];

        let mut response = Message::new(
            MessageType::SendData,
            self.data.local_time,
            self.data.address,
            self.data.local_time + self.message_latency(),
            msg.content.target_node.unwrap(),
        );
        response.content.data = Some(
            self.shares
                .get(
                    self.data
                        .tree_node
                        .parents
                        .iter()
                        .position(|&parent| parent == msg.content.target_node.unwrap())
                        .unwrap(),
                )
                .unwrap()
                .clone(),
        );

        resulting_messages.push(response);

        resulting_messages
    }
}
