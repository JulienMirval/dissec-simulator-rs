use std::collections::HashMap;

use crate::{
    common::Address,
    message::{Message, MessageType},
    run::RunSettings,
    shares::{AggregatableShares, Share},
    tree_node::TreeNode,
};

use super::{ChannelState, NodeRole};

#[derive(Debug)]
pub struct NodeData {
    pub settings: RunSettings,
    pub address: Address,
    pub role: NodeRole,
    pub local_time: f64,
    pub death_time: f64,
    pub opened_channels: Vec<ChannelState>,
    pub tree_node: TreeNode,
    pub finished_working: bool,
    pub aggregates: HashMap<Address, Share>,
    pub secret_value: f64,
}

pub trait Node {
    fn new(settings: RunSettings, address: Address) -> Box<Self>
    where
        Self: Sized;

    fn data(&self) -> &NodeData;
    fn data_mut(&mut self) -> &mut NodeData;

    fn setup(&mut self, current_time: f64) -> Vec<Message> {
        let mut messages = vec![];

        messages.push(Message::new(
            MessageType::ScheduleHealthCheck,
            current_time,
            self.data().address,
            current_time,
            self.data().address,
        ));

        messages
    }

    fn handle_message(&mut self, msg: &mut Message) -> Option<Vec<Message>> {
        if self.data().death_time <= msg.arrival_time {
            // The node is dead by the time the message arrives
            return Some(vec![]);
        }
        if msg.arrival_time < self.data().local_time
            && msg.message_type != MessageType::RequestHealth
        {
            msg.arrival_time = self.data().local_time;
            return None;
        }

        self.data_mut().local_time = if self.data().local_time < msg.arrival_time {
            msg.arrival_time
        } else {
            self.data().local_time
        };

        print!(
            "[@{}] Node #{} ({}): ",
            self.data().local_time,
            self.data().address,
            self.data().role
        );

        let time_before: f64 = self.data().local_time;
        msg.delivered = true;
        let resulting_messages = match msg.message_type {
            MessageType::ScheduleHealthCheck => self.handle_schedule_health_check(msg),
            MessageType::RequestHealth => self.handle_request_health(msg),
            MessageType::ConfirmHealth => self.handle_confirm_health(msg),
            MessageType::RequestData => self.handle_request_data(msg),
            MessageType::PrepareData => self.handle_prepare_data(msg),
            MessageType::SendData => self.handle_send_data(msg),
            MessageType::OpenChannel => self.handle_open_channel(msg),
            MessageType::ConfirmChannel => self.handle_confirm_channel(msg),
            t => panic!("Unknown message type: {}", t),
        };

        // Work of the message = time spent working by the node
        msg.work = self.data().local_time - time_before;

        Some(resulting_messages)
    }
    fn handle_schedule_health_check(&mut self, msg: &mut Message) -> Vec<Message> {
        println!("Node #{} is sending health checks", msg.emitter);
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
                    self.data().local_time + self.message_latency(),
                    channel.peer_address,
                ))
            });

        // Reschedule
        resulting_messages.push(Message::new_timeout(
            MessageType::ScheduleHealthCheck,
            self.data().address,
            self.data().local_time,
            self.data().local_time + self.data().settings.health_check_period,
        ));

        resulting_messages
    }
    fn handle_request_health(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received a health check from node #{}",
            msg.receiver, msg.emitter
        );
        let mut resulting_messages = vec![];

        resulting_messages.push(Message::new(
            MessageType::ConfirmHealth,
            self.data().local_time,
            self.data().address,
            self.data().local_time + self.message_latency(),
            msg.emitter,
        ));

        resulting_messages
    }
    fn handle_confirm_health(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received a health confirmation from node #{}",
            msg.receiver, msg.emitter
        );
        let resulting_messages = vec![];

        resulting_messages
    }
    fn handle_open_channel(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received channel opening request from node #{}",
            msg.receiver, msg.emitter
        );
        let mut resulting_messages = vec![];

        self.data_mut()
            .opened_channels
            .push(ChannelState::new(msg.emitter, true));
        self.data_mut().local_time += 3.0 * self.data().settings.costs.crypto;
        resulting_messages.push(Message::new(
            MessageType::ConfirmChannel,
            self.data().local_time,
            self.data().address,
            self.data().local_time + self.message_latency(),
            msg.emitter,
        ));

        resulting_messages
    }
    fn handle_confirm_channel(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received the confirmation of a channel opening #{}",
            msg.receiver, msg.emitter
        );
        let resulting_messages = vec![];

        self.data_mut()
            .opened_channels
            .push(ChannelState::new(msg.emitter, true));
        self.data_mut().local_time += 3.0 * self.data().settings.costs.crypto;

        resulting_messages
    }
    fn handle_request_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} received a data request #{}",
            msg.receiver, msg.emitter
        );
        let resulting_messages = vec![];

        resulting_messages
    }
    fn handle_prepare_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} is preparing to send data to node #{:?}",
            msg.receiver, msg.content.target_node
        );
        let resulting_messages = vec![];

        resulting_messages
    }
    fn handle_send_data(&mut self, msg: &mut Message) -> Vec<Message> {
        println!(
            "Node #{} is receiving data from node #{}",
            msg.receiver, msg.emitter
        );
        let mut resulting_messages = vec![];

        let aggregate = msg.content.data.clone().unwrap();
        self.data_mut().aggregates.insert(msg.emitter, aggregate);

        let expected_data = self
            .data()
            .tree_node
            .children
            .iter()
            .map(|child_group| {
                // HACK: This handler should be implemented for each role
                if self.data().role == NodeRole::LeafAggregator {
                    // The child is a contributor
                    self.data().aggregates.get(&child_group[0])
                } else {
                    let position = self
                        .data()
                        .tree_node
                        .members
                        .iter()
                        .position(|&member| self.data().address == member)
                        .unwrap();
                    self.data().aggregates.get(&child_group[position])
                }
            })
            .collect::<Vec<_>>();
        let received_all_data = expected_data.iter().all(|data| data.is_some());
        if received_all_data {
            let position = self
                .data()
                .tree_node
                .members
                .iter()
                .position(|&member| self.data().address == member)
                .unwrap();
            let mut msg = Message::new(
                MessageType::SendData,
                self.data().local_time,
                self.data().address,
                self.data().local_time + self.message_latency(),
                *self.data().tree_node.parents.get(position).unwrap(),
            );
            msg.content.data = Some(
                expected_data
                    .iter()
                    .map(|&x| x.unwrap().clone())
                    .collect::<Vec<_>>()
                    .aggregate(),
            );
            resulting_messages.push(msg);
        }

        resulting_messages
    }

    fn message_latency(&self) -> f64 {
        self.data().settings.costs.comm
    }
}
