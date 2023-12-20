/*!
The local-gateway crate provides a local backend for Grafbase developer tools

# Example

```ignore
use backend::server_api::start_server;
# common::environment::Environment::try_init().unwrap();

const PORT: Option<u16> = Some(4000);
const SEARCH: bool = true;

// `common::environment::Environment` must be initialized before this

let (server_port, server_handle) = start_server(PORT, SEARCH).unwrap();
```
*/

// TODO: make the prior example testable

#![forbid(unsafe_code)]

mod consts;

pub mod api;
pub mod errors;
pub mod project;
pub mod types;
