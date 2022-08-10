#[allow(clippy::doc_markdown)] // false positive
/// SQLite error codes as defined by <https://www.sqlite.org/rescode.html>
///
/// SQLite extended error codes are normally `i32`, but due to [`sqlx`] returning them as `Cow<'_, &str>` the codes here are defined as
/// `&str`s for convenience.
pub mod extended_error_codes {
    /// The `SQLITE_CONSTRAINT_PRIMARYKEY` error code is an extended error code for `SQLITE_CONSTRAINT` indicating that a PRIMARY KEY constraint failed.
    pub const SQLITE_CONSTRAINT_PRIMARYKEY: &str = "1555";
}
