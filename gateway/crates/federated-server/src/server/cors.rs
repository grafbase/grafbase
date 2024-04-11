use crate::config::CorsConfig;
use tower_http::cors::CorsLayer;

pub(super) fn generate(cors: CorsConfig) -> CorsLayer {
    let CorsConfig {
        allow_credentials,
        allow_origins,
        max_age,
        allow_methods,
        allow_headers,
        expose_headers,
        allow_private_network,
    } = cors;

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
