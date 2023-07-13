use std::net::Ipv4Addr;

#[derive(Clone, Copy)]
pub enum LocalAddressType {
    /// 127.0.0.1
    Localhost,
    /// 0.0.0.0
    Unspecified,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UdfMessageLevel {
    Debug,
    Error,
    Info,
    Warn,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum UdfKind {
    Resolver,
    Authorizer,
}

// FIXME: remove after api repo is updated
impl Default for UdfKind {
    fn default() -> Self {
        Self::Resolver
    }
}

#[derive(serde::Deserialize, Clone, Copy, Debug, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}
