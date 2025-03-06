mod access_log;
mod authorization_context;
mod cache;
mod headers;
mod http_client;
mod nats;
mod shared_context;
mod token;

use super::wit::*;
use crate::state::WasiState;

impl Host for WasiState {}
