use std::net::IpAddr;

use engine_value::ConstValue;
use ipnet::IpNet;

use super::{DynamicParse, SDLDefinitionScalar};
use crate::{Error, InputValueError, InputValueResult};

pub struct IPAddressScalar;

impl IPAddressScalar {
    pub fn parse_value(value: serde_json::Value) -> Result<IpAddr, Error> {
        let ip = serde_json::from_value::<String>(value)?;
        Ok(ip
            .parse::<IpNet>()
            .map(|ip| ip.addr())
            .or_else(|_| ip.parse::<IpAddr>())?)
    }
}

impl<'a> SDLDefinitionScalar<'a> for IPAddressScalar {
    fn name() -> Option<&'a str> {
        Some("IPAddress")
    }

    fn description() -> Option<&'a str> {
        Some(
            r"A valid IPv4 or IPv6 address. IPv4 addresses are expected in quad-dotted notation `(123.12.34.56)`. IPv6 addresses are expected in non-bracketed, colon-separated format `(1a2b:3c4b::1234:4567)`.

You can include an optional CIDR suffix `(123.45.67.89/16)` to indicate subnet mask.",
        )
    }

    fn specified_by() -> Option<&'a str> {
        Some("https://tools.ietf.org/html/rfc4291")
    }
}

impl DynamicParse for IPAddressScalar {
    fn is_valid(value: &ConstValue) -> bool {
        match value {
            ConstValue::String(ip) => ip.parse::<IpNet>().is_ok() || ip.parse::<IpAddr>().is_ok(),
            _ => false,
        }
    }

    fn to_value(value: serde_json::Value) -> Result<ConstValue, Error> {
        match value {
            serde_json::Value::String(ip) => ip
                .parse::<IpNet>()
                .map(|ip| ConstValue::String(ip.to_string()))
                .or_else(|_| {
                    ip.parse::<IpAddr>()
                        .map(|x| ConstValue::String(x.to_string()))
                        .map_err(|_| Error::new("Cannot coerce the initial value into an IP address or an IP range."))
                }),
            _ => Err(Error::new(
                "Data violation: Cannot coerce the initial value into an IPAddress",
            )),
        }
    }

    fn parse(value: ConstValue) -> InputValueResult<serde_json::Value> {
        match value {
            ConstValue::String(ip) => ip
                .parse::<IpNet>()
                .map(|ip| serde_json::Value::String(ip.to_string()))
                .or_else(|_| {
                    ip.parse::<IpAddr>()
                        .map(|x| serde_json::Value::String(x.to_string()))
                        .map_err(|_| {
                            InputValueError::ty_custom(
                                "IPAddress",
                                "Cannot parse the value into an IP address or an IP range.",
                            )
                        })
                }),
            _ => Err(InputValueError::ty_custom("IPAddress", "Cannot parse into a IPAddress")),
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_value::ConstValue;
    use insta::assert_snapshot;

    use super::super::SDLDefinitionScalar;
    use crate::registry::scalars::{DynamicParse, IPAddressScalar};

    #[test]
    fn check_valid_ip_address_ipv4() {
        let value = serde_json::Value::String("127.0.0.1".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = IPAddressScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn check_valid_ip_address_ipv6() {
        let value = serde_json::Value::String("::1".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = IPAddressScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn check_valid_ip_range() {
        let value = serde_json::Value::String("123.45.67.89/16".to_string());

        let const_value = ConstValue::from_json(value).unwrap();

        let scalar = IPAddressScalar::parse(const_value);
        assert!(scalar.is_ok());
    }

    #[test]
    fn ensure_directives_sdl() {
        assert_snapshot!(IPAddressScalar::sdl());
    }
}
