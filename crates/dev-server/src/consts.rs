use std::ops::Range;

pub const WORKER_DIR: &str = "worker";
pub const WORKER_FOLDER_VERSION_FILE: &str = "version.txt";
pub const EPHEMERAL_PORT_RANGE: Range<u16> = 49152..65535;
pub const SCHEMA_PARSER_DIR: &str = "parser";
pub const SCHEMA_PARSER_INDEX: &str = "index.js";
pub const GIT_IGNORE_FILE: &str = ".gitignore";
pub const MIN_NODE_VERSION: &str = "v16.0.0";
