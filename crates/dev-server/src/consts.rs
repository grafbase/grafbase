use std::ops::Range;

pub const WORKER_DIR: &str = "worker";
pub const WORKER_FOLDER_VERSION_FILE: &str = "version.txt";
pub const EPHEMERAL_PORT_RANGE: Range<u16> = 49152..65535;
