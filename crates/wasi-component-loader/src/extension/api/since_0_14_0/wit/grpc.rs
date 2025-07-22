mod client;
mod streaming_response;

pub use super::grafbase::sdk::grpc::*;

use crate::InstanceState;

impl Host for InstanceState {}
