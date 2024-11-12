//! This crate implements operation checks, without all the IO parts. The intended workflow is the
//! following:
//!
//! - Parse the two schemas and convert them to [Schema] structs, and do the same for all the
//!   relevant [Operation]s.
//! - Aggregate field usage from the operations with [aggregate_field_usage()].
//! - Run the checks with [check()].

#![allow(unused_crate_dependencies)]
#![deny(missing_docs)]

mod aggregate_field_usage;
mod check;
mod operation;
mod schema;

pub use aggregate_field_usage::{aggregate_field_usage, FieldUsage};
pub use check::{check, CheckDiagnostic, CheckParams, Severity};
pub use operation::Operation;
pub use schema::Schema;
