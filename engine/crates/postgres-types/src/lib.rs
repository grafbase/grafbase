pub mod cursor;
pub mod database_definition;
pub mod error;
pub mod transport;

pub type Result<T> = std::result::Result<T, error::Error>;
