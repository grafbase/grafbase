/*!
The local-gateway crate provides a local backend for Grafbase developer tools

# Example

```ignore
use local_gateway::dev_server_api::start_dev_server;
# common::environment::Environment::try_init().unwrap();

const PORT: Option<u16> = Some(4000);
const SEARCH: bool = true;

// `common::environment::Environment` must be initialized before this

let (dev_server_port, dev_server_handle) = start_dev_server(PORT, SEARCH).unwrap();
```
*/

// TODO: make the prior example testable

#![forbid(unsafe_code)]

pub mod dev_server_api;
pub mod errors;
pub mod types;
