use ascii::AsciiString;
use duration_str::deserialize_option_duration;
use std::time::Duration;
use url::Url;

#[derive(Clone, Default, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CorsConfig {
    /// If false (or not defined), credentials are not allowed in requests
    pub allow_credentials: bool,
    /// Origins from which we allow requests
    pub allow_origins: Option<AnyOrUrlArray>,
    /// Maximum time between OPTIONS and the next request
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub max_age: Option<Duration>,
    /// HTTP methods allowed to the endpoint.
    pub allow_methods: Option<AnyOrHttpMethodArray>,
    /// Headers allowed in incoming requests
    pub allow_headers: Option<AnyOrAsciiStringArray>,
    /// Headers exposed from the OPTIONS request
    pub expose_headers: Option<AnyOrAsciiStringArray>,
    /// If set, allows browsers from private network to connect
    pub allow_private_network: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Deserialize, strum::EnumString)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl From<http::Method> for HttpMethod {
    fn from(value: http::Method) -> Self {
        if value == http::Method::GET {
            Self::Get
        } else if value == http::Method::POST {
            Self::Post
        } else if value == http::Method::PUT {
            Self::Put
        } else if value == http::Method::DELETE {
            Self::Delete
        } else if value == http::Method::PATCH {
            Self::Patch
        } else if value == http::Method::HEAD {
            Self::Head
        } else if value == http::Method::OPTIONS {
            Self::Options
        } else if value == http::Method::TRACE {
            Self::Trace
        } else if value == http::Method::CONNECT {
            Self::Connect
        } else {
            todo!("Unsupported HTTP method: {:?}", value);
        }
    }
}

impl From<HttpMethod> for http::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Head => http::Method::HEAD,
            HttpMethod::Options => http::Method::OPTIONS,
            HttpMethod::Connect => http::Method::CONNECT,
            HttpMethod::Patch => http::Method::PATCH,
            HttpMethod::Trace => http::Method::TRACE,
        }
    }
}

pub type AnyOrUrlArray = AnyOrArray<Url>;

pub type AnyOrHttpMethodArray = AnyOrArray<HttpMethod>;

pub type AnyOrAsciiStringArray = AnyOrArray<AsciiString>;

#[derive(Clone, Debug, PartialEq)]
pub enum AnyOrArray<T> {
    Any,
    Explicit(Vec<T>),
}

impl<'de, T> serde::Deserialize<'de> for AnyOrArray<T>
where
    T: serde::Deserialize<'de> + std::str::FromStr<Err: std::fmt::Display>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct AnyOrArrayVisitor<T> {
            _marker: std::marker::PhantomData<T>,
        }

        impl<'de, T> serde::de::Visitor<'de> for AnyOrArrayVisitor<T>
        where
            T: serde::Deserialize<'de> + std::str::FromStr<Err: std::fmt::Display>,
        {
            type Value = AnyOrArray<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("expecting string \"*\", or an array of values")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value == "*" || value == "any" {
                    Ok(AnyOrArray::Any)
                } else {
                    value
                        .parse::<T>()
                        .map_err(|err| E::custom(err))
                        .map(|value| AnyOrArray::Explicit(vec![value]))
                }
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut array = Vec::new();
                while let Some(value) = seq.next_element()? {
                    array.push(value);
                }
                Ok(AnyOrArray::Explicit(array))
            }
        }

        deserializer.deserialize_any(AnyOrArrayVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}
