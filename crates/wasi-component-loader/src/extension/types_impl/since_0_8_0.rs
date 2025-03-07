use crate::WasiState;

mod access_log;
mod cache;
mod headers;
mod http_client;
mod nats;
mod shared_context;
mod token;

impl crate::wit::since_0_8_0::grafbase::sdk::access_log::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::cache::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::context::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::error::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::headers::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::http_client::Host for WasiState {}
impl crate::wit::since_0_8_0::grafbase::sdk::nats_client::Host for WasiState {}
