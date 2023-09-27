pub use common_types::UdfKind;
use std::net::Ipv4Addr;

#[derive(Clone, Copy)]
pub enum LocalAddressType {
    /// 127.0.0.1
    Localhost,
    /// 0.0.0.0
    Unspecified,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LocalAddressType {
    #[must_use]
    pub const fn to_ip_v4(&self) -> Ipv4Addr {
        match self {
            Self::Localhost => Ipv4Addr::LOCALHOST,
            Self::Unspecified => Ipv4Addr::UNSPECIFIED,
        }
    }
}

#[derive(serde::Deserialize, Clone, Copy, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}
