//! This crate implements operation checks, without all the IO parts. The intended workflow is the
//! following:
//!
//! - Parse the two schemas and convert them to [Schema] structs, and do the same for all the
//!   relevant [Operation]s.
//! - Aggregate field usage from the operations with [aggregate_field_usage()].
//! - Run the checks with [check()].
//! - Alternatively, you can use [check_assuming_all_used()] which assumes all fields, arguments,
//!   and enum values are in use.

#![deny(missing_docs)]

mod aggregate_field_usage;
mod check;
mod operation;
mod schema;

pub use aggregate_field_usage::{AssumeAllUsed, FieldUsage, UsageProvider, aggregate_field_usage};
pub use check::{CheckDiagnostic, CheckParams, Severity, check, check_assuming_all_used};
pub use operation::Operation;
pub use schema::Schema;
