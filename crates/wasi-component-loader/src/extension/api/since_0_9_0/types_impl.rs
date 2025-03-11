use crate::WasiState;

pub mod access_log;
pub mod authorization_context;
pub mod cache;
pub mod error;
pub mod headers;
pub mod http_client;
pub mod nats;
pub mod shared_context;

impl super::wit::grafbase::sdk::access_log::Host for WasiState {}
impl super::wit::grafbase::sdk::cache::Host for WasiState {}
impl super::wit::grafbase::sdk::context::Host for WasiState {}
impl super::wit::grafbase::sdk::error::Host for WasiState {}
impl super::wit::grafbase::sdk::headers::Host for WasiState {}
impl super::wit::grafbase::sdk::http_client::Host for WasiState {}
impl super::wit::grafbase::sdk::nats_client::Host for WasiState {}
impl super::wit::grafbase::sdk::token::Host for WasiState {}
