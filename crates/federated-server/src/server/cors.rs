use gateway_config::{AnyOrAsciiStringArray, AnyOrHttpMethodArray, AnyOrUrlArray, CorsConfig};
use http::{HeaderName, HeaderValue};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};

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
    }: &CorsConfig,
) -> CorsLayer {
    let mut cors_layer = CorsLayer::new()
        .allow_credentials(*allow_credentials)
        .allow_private_network(*allow_private_network);

    if let Some(allow_origins) = allow_origins {
        cors_layer = cors_layer.allow_origin(match allow_origins {
            AnyOrUrlArray::Any => AllowOrigin::any(),
            AnyOrUrlArray::Explicit(origins) => {
                let mut constants = Vec::new();
                let mut globs = Vec::new();
                for origin in origins {
                    let origin = &origin[..url::Position::BeforePath];
                    if origin.chars().any(|c| "?*[]{}!\\".contains(c)) {
                        globs.push(origin.to_owned());
                    } else {
                        constants.push(HeaderValue::from_str(origin).expect("must be ascii"));
                    }
                }
                if globs.is_empty() {
                    AllowOrigin::list(constants)
                } else if constants.is_empty() {
                    AllowOrigin::predicate(move |origin, _| -> bool {
                        for glob in &globs {
                            if fast_glob::glob_match(glob, origin) {
                                return true;
                            }
                        }

                        false
                    })
                } else {
                    AllowOrigin::predicate(move |origin, _| -> bool {
                        for constant in &constants {
                            if origin == constant {
                                return true;
                            }
                        }

                        for glob in &globs {
                            if fast_glob::glob_match(glob, origin) {
                                return true;
                            }
                        }

                        false
                    })
                }
            }
        });
    }

    if let Some(max_age) = max_age {
        cors_layer = cors_layer.max_age(*max_age);
    }

    if let Some(allow_methods) = allow_methods {
        cors_layer = cors_layer.allow_methods(match allow_methods {
            AnyOrHttpMethodArray::Any => AllowMethods::any(),
            AnyOrHttpMethodArray::Explicit(methods) => {
                let methods = methods.iter().map(|method| http::Method::from(*method));
                AllowMethods::list(methods)
            }
        });
    }

    if let Some(allow_headers) = allow_headers {
        cors_layer = cors_layer.allow_headers(match allow_headers {
            AnyOrAsciiStringArray::Any => AllowHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                AllowHeaders::list(headers)
            }
        });
    }

    if let Some(expose_headers) = expose_headers {
        cors_layer = cors_layer.expose_headers(match expose_headers {
            AnyOrAsciiStringArray::Any => ExposeHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                ExposeHeaders::list(headers)
            }
        });
    }

    cors_layer
}
