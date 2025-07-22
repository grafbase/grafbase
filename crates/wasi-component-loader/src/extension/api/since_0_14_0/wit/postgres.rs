mod connection;
mod pool;

pub use super::grafbase::sdk::postgres::*;

use crate::InstanceState;

impl Host for InstanceState {}
