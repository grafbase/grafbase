use std::sync::atomic::AtomicU16;

pub static WORKER_PORT: AtomicU16 = AtomicU16::new(0);
