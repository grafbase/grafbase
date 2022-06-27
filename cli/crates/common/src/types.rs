use std::net::Ipv4Addr;

#[derive(Clone, Copy)]
pub enum LocalAddressType {
    /// 127.0.0.1
    Localhost,
    /// 0.0.0.0
    Unspecified,
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
