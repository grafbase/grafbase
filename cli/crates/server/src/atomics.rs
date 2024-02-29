use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64};

pub static WORKER_PORT: AtomicU16 = AtomicU16::new(0);
pub static REGISTRY_PARSED_EPOCH_OFFSET_MILLIS: AtomicU64 = AtomicU64::new(0);
/// allows to rerun the bun installation without rechecking
/// external binaries to match the version
pub static BUN_INSTALLED_FOR_SESSION: AtomicBool = AtomicBool::new(false);
