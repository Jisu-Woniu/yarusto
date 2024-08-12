use std::{path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::{
    error::Result,
    model::{
        config::Config,
        types::{
            duration::Duration,
            judge::{Case, ResourceLimits, TaskType},
            memory_size::MemorySize,
        },
    },
};

#[derive(Debug, Deserialize)]
pub struct RawConfig1 {
    #[serde(default)]
    time: Duration,
    #[serde(default)]
    memory: MemorySize,
}

impl Config for RawConfig1 {
    fn resource_limits(&self) -> Result<ResourceLimits> {
        Ok(ResourceLimits {
            time: self.time.as_millis(),
            memory: self.memory.as_kib(),
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
