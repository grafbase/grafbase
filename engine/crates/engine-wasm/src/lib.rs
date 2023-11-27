#![allow(unused_crate_dependencies)]

mod gateway;
mod pg;

use gateway::PgCallbacks;

#[derive(serde::Deserialize)]
struct RegistryWithVersion {
    cli_version: String,
    registry: engine::Registry,
}
