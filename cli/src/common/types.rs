#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "UPPERCASE")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(serde::Deserialize, Clone, Copy, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}
