pub const DATABASE_FILE: &str = "data.sqlite";
pub const DATABASE_URL_PREFIX: &str = "sqlite://";
pub const PREPARE: &str = include_str!("../sql/prepare.sql");

pub const DYNAMODB_PK: &str = "__pk";
pub const DYNAMODB_SK: &str = "__sk";
pub const DYNAMODB_TYPE_INDEX_PK: &str = "__gsi1pk";
pub const DYNAMODB_TYPE_INDEX_SK: &str = "__gsi1sk";
pub const DYNAMODB_INVERTED_INDEX_PK: &str = "__gsi2pk";
pub const DYNAMODB_INVERTED_INDEX_SK: &str = "__gsi2sk";
