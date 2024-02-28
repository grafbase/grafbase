/*!
The common crate provides shared functionality for Grafbase developer tools
*/

#![forbid(unsafe_code)]

#[cfg(not(test))]
use expect_test as _;

pub mod analytics;
pub mod channels;
pub mod consts;
pub mod debug_macros;
pub mod environment;
pub mod errors;
pub mod trusted_documents;
pub mod types;
pub mod utils;
