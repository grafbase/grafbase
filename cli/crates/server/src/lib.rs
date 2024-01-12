/*!
The server crate provides a server with the gateway worker
and a bridge server

# Example

```ignore
const PORT: u16 = 4000;
# common::environment::Environment::try_init().unwrap();

// `common::environment::Environment` must be initialized before this

let server_handle = server::start(PORT).unwrap();
```
*/

// TODO: make the prior example testable

#![forbid(unsafe_code)]

#[macro_use]
extern crate log;

mod atomics;
mod bridge;
mod codegen_server;
mod config;
mod consts;
mod dump_config;
mod environment;
mod error_server;
mod file_watcher;
mod introspect_local;
mod node;
mod proxy;
mod servers;
mod udf_builder;

pub mod errors;
pub mod types;

pub use dump_config::dump_config;
pub use introspect_local::{introspect_local, IntrospectLocalOutput};
pub use servers::{export_embedded_files, start, PortSelection, ProductionServer};
