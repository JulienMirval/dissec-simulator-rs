use std::{
    error::Error,
    fs::{self, File},
};

use crate::{message::Message, run::RunSettings};

#[derive(Debug)]
pub struct Recording {
    pub settings: RunSettings,
    pub total_latency: f64,
    pub total_work: f64,
    pub total_bandwidth: f64,
    pub initial_contributors: usize,
    pub final_contributors: usize,
    pub sent_messages: Vec<Message>,
    pub full_export: bool,
}

impl Recording {
    pub fn new(settings: RunSettings, full_export: bool) -> Recording {
        Recording {
            settings,
            total_latency: 0.0,
            total_work: 0.0,
            total_bandwidth: 0.0,
            initial_contributors: 0,
            final_contributors: 0,
            sent_messages: vec![],
            full_export,
        }
    }

    pub fn record(&mut self, msg: &Message) {
        if self.full_export {
            self.sent_messages.push(msg.clone());
            self.total_work += msg.work;
            self.total_latency = msg.arrival_time;
            if msg.content.data.is_some() {
                self.total_bandwidth += 1.0;
            };
        } else {
            self.total_work += msg.work;
            self.total_latency = msg.arrival_time;
            if msg.content.data.is_some() {
                self.total_bandwidth += 1.0;
            };
        }
    }

    pub fn write_to_path(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let path = format!("outputs/{}", filename);
        match fs::create_dir("outputs") {
            _ => (),
        };
        let mut writter = match csv::Writer::from_path(path.as_str()) {
            Result::Ok(writter) => writter,
            Result::Err(_) => {
                File::create(path.as_str())?;
                csv::Writer::from_path(path.as_str())?
            }
        };

        let columns = [
            "seed",
            "failure_handling",
            "average_failure_time",
            "health_check_period",
            "communication_cost",
            "crypto_cost",
            "compute_cost",
            "tree_depth",
            "tree_fanout",
            "group_size",
            "total_work",
            "total_latency",
            "total_bandwidth",
            "completeness",
            "message_type",
            "emitter_address",
            "receiver_address",
            "departure_time",
            "arrival_time",
        ];

        let completeness = ((self.initial_contributors - self.final_contributors) as f64
            / self.initial_contributors as f64)
            .to_string();
        writter.write_record(&columns)?;
        if self.full_export {
            for msg in &self.sent_messages {
                writter.write_record(&[
                    self.settings.seed.clone(),
                    self.settings.building_blocks.failure_handling.to_string(),
                    self.settings.average_failure_time.to_string(),
                    self.settings.health_check_period.to_string(),
                    self.settings.costs.comm.to_string(),
                    self.settings.costs.crypto.to_string(),
                    self.settings.costs.compute.to_string(),
                    self.settings.tree.depth.to_string(),
                    self.settings.tree.fanout.to_string(),
                    self.settings.tree.group_size.to_string(),
                    self.total_work.to_string(),
                    self.total_latency.to_string(),
                    self.total_bandwidth.to_string(),
                    completeness.to_string(),
                    msg.message_type.to_string(),
                    msg.emitter.to_string(),
                    msg.receiver.to_string(),
                    msg.departure_time.to_string(),
                    msg.arrival_time.to_string(),
                ])?;
            }
        } else {
            writter.write_record(&[
                self.settings.seed.clone(),
                self.settings.building_blocks.failure_handling.to_string(),
                self.settings.average_failure_time.to_string(),
                self.settings.health_check_period.to_string(),
                self.settings.costs.comm.to_string(),
                self.settings.costs.crypto.to_string(),
                self.settings.costs.compute.to_string(),
                self.settings.tree.depth.to_string(),
                self.settings.tree.fanout.to_string(),
                self.settings.tree.group_size.to_string(),
                self.total_work.to_string(),
                self.total_latency.to_string(),
                self.total_bandwidth.to_string(),
                completeness.to_string(),
                "Stop".to_string(),
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
                "0".to_string(),
            ])?;
        }
        writter.flush()?;

        Ok(())
    }
}
