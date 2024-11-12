/*!
The common crate provides shared functionality for Grafbase developer tools
*/

#![forbid(unsafe_code)]

use grafbase_workspace_hack as _;

#[cfg(not(test))]
use expect_test as _;

pub mod channels;
pub mod consts;
pub mod debug_macros;
pub mod environment;
pub mod errors;
pub mod pathfinder;
pub mod trusted_documents;
pub mod types;
pub mod utils;
