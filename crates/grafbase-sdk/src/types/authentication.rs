use crate::wit;

use super::OwnedHttpHeaders;

/// An HTTP endpoint exposed publicly on the Gateway. This is typically used to return metadata for authentication purposes, for example with the [OAuth 2.0 Protected Resource Metadata](https://datatracker.ietf.org/doc/html/rfc9728) spec.
#[non_exhaustive]
pub struct PublicMetadataEndpoint {
    /// The absolute path (without domain) of the endpoint. Example: "/.well-known/oauth-protected-resource".
    path: String,
    /// The contents of the response body that the endpoint will return. Example: '{"resource": "https://secure.example.com" }'.
    response_body: Vec<u8>,
    /// The headers sent from with the response by the public endpoint. Example: ["Content-Type: application/json"].
    response_headers: OwnedHttpHeaders,
}

impl PublicMetadataEndpoint {
    /// Constructor.
    pub fn new(path: String, response_body: Vec<u8>) -> Self {
        PublicMetadataEndpoint {
            path,
            response_body,
            response_headers: Default::default(),
        }
    }

    /// Set the response headers
    pub fn with_headers(mut self, response_headers: OwnedHttpHeaders) -> Self {
        self.response_headers = response_headers;
        self
    }

    /// Access the response headers
    pub fn response_headers_mut(&mut self) -> &mut OwnedHttpHeaders {
        &mut self.response_headers
    }
}

impl From<wit::PublicMetadataEndpoint> for PublicMetadataEndpoint {
    fn from(
        wit::PublicMetadataEndpoint {
            path,
            response_body,
            response_headers,
        }: wit::PublicMetadataEndpoint,
    ) -> Self {
        PublicMetadataEndpoint {
            path,
            response_body,
            response_headers: response_headers.into(),
        }
    }
}

impl From<PublicMetadataEndpoint> for wit::PublicMetadataEndpoint {
    fn from(
        PublicMetadataEndpoint {
            path,
            response_body,
            response_headers,
        }: PublicMetadataEndpoint,
    ) -> Self {
        wit::PublicMetadataEndpoint {
            path,
            response_body,
            response_headers: response_headers.into(),
        }
    }
}
