use crate::RequestHeaders;

pub fn build_connector_header_vec<'a>(
    connector_headers: &'a registry_v2::ConnectorHeaders,
    request_headers: &'a RequestHeaders,
) -> Vec<(&'a str, &'a str)> {
    connector_headers
        .0
        .iter()
        .filter_map(|(name, value)| match value {
            registry_v2::ConnectorHeaderValue::Static(static_value) => Some((name.as_str(), static_value.as_str())),
            registry_v2::ConnectorHeaderValue::Forward(header_name) => {
                Some((name.as_str(), request_headers.find(header_name)?))
            }
        })
        .collect()
}
