use std::{
    error::Error,
    fs::{self, File},
};

use crate::{
    message::Message,
    run::{BuildingBlocks, FailureHandlingMode},
};

#[derive(Debug)]
pub struct Recording {
    pub failure_handling_mode: FailureHandlingMode,
    pub latency: f64,
    pub sent_messages: Vec<Message>,
    pub full_export: bool,
}

impl Recording {
    pub fn new(building_blocks: BuildingBlocks, full_export: bool) -> Recording {
        Recording {
            failure_handling_mode: building_blocks.failure_handling,
            latency: 0.0,
            sent_messages: vec![],
            full_export,
        }
    }

    pub fn record(&mut self, msg: &Message) {
        if self.full_export {
            self.sent_messages.push(msg.clone());
        } else {
            self.latency = msg.arrival_time;
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

        let columns = ["failure_handling_mode", "departure_time", "arrival_time"];

        writter.write_record(&columns)?;
        if self.full_export {
            for msg in &self.sent_messages {
                writter.write_record(&[
                    format!("{}", self.failure_handling_mode),
                    format!("{}", msg.departure_time),
                    format!("{}", msg.arrival_time),
                ])?;
            }
        } else {
            writter.write_record(&[
                format!("{}", self.failure_handling_mode),
                format!("{}", 0),
                format!("{}", self.latency),
            ])?;
        }
        writter.flush()?;

        Ok(())
    }
}
