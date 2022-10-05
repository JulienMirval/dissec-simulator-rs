use crate::{
    common::Address,
    message::{Message, MessageType},
    run::RunSettings,
    tree_node::TreeNode,
};

use super::{ChannelState, NodeRole};

pub struct NodeData {
    pub settings: RunSettings,
    pub address: Address,
    pub role: NodeRole,
    pub local_time: f64,
    pub death_time: f64,
    pub opened_channels: Vec<ChannelState>,
    pub tree_node: TreeNode,
}

pub trait Node {
    fn new(settings: RunSettings, address: Address) -> Box<Self>
    where
        Self: Sized;

    fn settings(&self) -> &RunSettings;
    fn data(&self) -> &NodeData;
    fn data_mut(&mut self) -> &mut NodeData;

    fn initialize(&mut self) {}

    fn handle_message(&mut self, msg: &mut Message) -> Vec<Message> {
        match msg.message_type {
            MessageType::ScheduleHealthCheck => self.handle_schedule_health_check(msg),
            MessageType::RequestHealth => self.handle_request_health(msg),
            MessageType::ConfirmHealth => self.handle_confirm_health(msg),
            _ => panic!("Unknown message type"),
        }
    }
    fn handle_schedule_health_check(&mut self, msg: &mut Message) -> Vec<Message> {
        println!("Default implementation schedule HC");
        let mut resulting_messages = vec![];

        // Check maintained channels
        self.data()
            .opened_channels
            .iter()
            .filter(|&channel| channel.maintained)
            .for_each(|channel| {
                resulting_messages.push(Message::new(
                    MessageType::RequestHealth,
                    self.data().local_time,
                    self.data().address,
                    channel.peer_address,
                ))
            });

        // Reschedule
        resulting_messages.push(Message::new_timeout(
            MessageType::ScheduleHealthCheck,
            self.data().address,
            self.data().local_time,
            self.data().local_time,
        ));

        resulting_messages
    }
    fn handle_request_health(&mut self, msg: &mut Message) -> Vec<Message> {
        println!("Default implementation request HC");
        let mut resulting_messages = vec![];

        resulting_messages
    }
    fn handle_confirm_health(&mut self, msg: &mut Message) -> Vec<Message> {
        println!("Default implementation send data");
        let mut resulting_messages = vec![];

        resulting_messages
    }
}
