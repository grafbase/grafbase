use std::sync::atomic::{AtomicU16, AtomicU64};

pub static WORKER_PORT: AtomicU16 = AtomicU16::new(0);
pub static REGISTRY_PARSED_EPOCH_OFFSET_MILLIS: AtomicU64 = AtomicU64::new(0);
