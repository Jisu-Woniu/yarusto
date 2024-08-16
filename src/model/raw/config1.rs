use std::{path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::{
    error::Result,
    model::{
        config::Config,
        types::{
            duration::CustomDuration,
            judge::{Case, ResourceLimits, TaskType},
            memory_size::MemorySize,
        },
    },
};

#[derive(Debug, Deserialize)]
pub struct ConfigData {
    #[serde(default)]
    time: CustomDuration,
    #[serde(default)]
    memory: MemorySize,
}

impl Config for ConfigData {
    fn resource_limits(&self) -> Result<ResourceLimits> {
        Ok(ResourceLimits {
            time: u32::try_from(self.time.as_millis())?,
            memory: self.memory.as_kibibyte(),
        })
    }

    fn task(&self) -> Result<TaskType> {
        Ok(TaskType::Simple {
            cases: vec![Case {
                input: PathBuf::from_str("1.in")?,
                answer: PathBuf::from_str("1.ans")?,
                score: None,
            }],
        })
    }
}
