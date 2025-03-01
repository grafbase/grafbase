mod access_log;
mod cache;
mod headers;
mod http_client;
mod nats;
mod shared_context;

use super::wit::*;
use crate::state::WasiState;

impl Host for WasiState {}
