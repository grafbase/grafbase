use crate::WasiState;

mod access_log;
mod cache;
mod error;
mod field_output;
mod headers;
mod http_client;
mod nats;
mod shared_context;

impl super::wit::grafbase::sdk::types::Host for WasiState {}
