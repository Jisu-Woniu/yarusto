use thiserror::Error;

pub mod duration;
pub mod judge;
pub mod memory_size;

#[derive(Debug, Error)]
#[error("unexpected unit: {0}")]
pub struct InvalidUnit(String);
