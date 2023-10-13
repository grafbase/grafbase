use engine::registry::ConnectorHeaders;
use url::Url;

pub struct GraphQlConnectorMetadata<'a> {
    pub(crate) name: &'a str,
    pub(crate) namespace: bool,
    pub(crate) url: &'a Url,
    pub(crate) headers: ConnectorHeaders,
    pub(crate) introspection_headers: Vec<(&'a str, &'a str)>,
    pub(crate) type_prefix: Option<&'a str>,
}
