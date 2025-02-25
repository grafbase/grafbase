use grafbase_sdk::serde::{Deserialize, Serialize};
use grafbase_sdk::types::Arguments;

#[derive(Debug)]
pub struct SnowflakeConnection {
    pub subgraph_name: String,
    pub args: SnowflakeConnectionArgs,
}

#[derive(Debug, Deserialize)]
pub struct SnowflakeConnectionArgs {
    pub name: String,
    pub account: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub schema: String,
    pub warehouse: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Snowflake {
    pub connection: String,
    pub query: String,
    #[serde(default)]
    pub params: Vec<serde_json::Value>,
}

impl SnowflakeConnection {
    pub fn from_directive(
        subgraph_name: String,
        args: Arguments,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            subgraph_name,
            args: args.deserialize()?,
        })
    }
} 