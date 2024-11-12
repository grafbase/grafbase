use gateway_config::CorsConfig;
use tower_http::cors::CorsLayer;

/// Generates a CORS layer based on the provided configuration.
///
/// # Arguments
///
/// - `CorsConfig`: A configuration struct that contains the following fields:
///   - `allow_credentials`: Indicates whether to allow credentials.
///   - `allow_origins`: A list of origins that are allowed to access the resource.
///   - `max_age`: The maximum age for the preflight request.
///   - `allow_methods`: A list of HTTP methods that are allowed.
///   - `allow_headers`: A list of headers that are allowed.
///   - `expose_headers`: A list of headers that can be exposed to the client.
///   - `allow_private_network`: Indicates whether to allow requests from private networks.
///
/// # Returns
///
/// Returns a `CorsLayer` configured with the specified settings.
pub(super) fn generate(
    CorsConfig {
        allow_credentials,
        allow_origins,
        max_age,
        allow_methods,
        allow_headers,
        expose_headers,
        allow_private_network,
    }: CorsConfig,
) -> CorsLayer {
    let mut cors_layer = CorsLayer::new()
        .allow_credentials(allow_credentials)
        .allow_private_network(allow_private_network);

    if let Some(allow_origins) = allow_origins {
        cors_layer = cors_layer.allow_origin(allow_origins);
    }

    if let Some(max_age) = max_age {
        cors_layer = cors_layer.max_age(max_age);
    }

    if let Some(allow_methods) = allow_methods {
        cors_layer = cors_layer.allow_methods(allow_methods);
    }

    if let Some(allow_headers) = allow_headers {
        cors_layer = cors_layer.allow_headers(allow_headers);
    }

    if let Some(expose_headers) = expose_headers {
        cors_layer = cors_layer.expose_headers(expose_headers);
    }

    cors_layer
}
