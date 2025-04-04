mod client;
mod streaming_response;

pub use super::grafbase::sdk::grpc::*;

use crate::WasiState;

impl Host for WasiState {}
