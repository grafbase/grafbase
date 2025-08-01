use mediatype::MediaType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseFormat {
    Complete(CompleteResponseFormat),
    Streaming(StreamingResponseFormat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompleteResponseFormat {
    /// Follow the [GraphQL over HTTP spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
    Json,
    /// Follow the [GraphQL over HTTP spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/spec/GraphQLOverHTTP.md
    GraphqlResponseJson,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamingResponseFormat {
    /// Follow the [incremental delivery spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/IncrementalDelivery.md
    IncrementalDelivery,
    /// Follow the [GraphQL over SSE spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverSSE.md
    GraphQLOverSSE,
    /// Follow the [GraphQL over WebSocket spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverWebSocket.md
    GraphQLOverWebSocket,
}

mod content_types {
    pub const APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");
    pub const APPLICATION_CBOR: http::HeaderValue = http::HeaderValue::from_static("application/cbor");
    pub const APPLICATION_GRAPHQL_RESPONSE_JSON: http::HeaderValue =
        http::HeaderValue::from_static("application/graphql-response+json");

    pub static SUPPORTED: [http::HeaderValue; 2] = [APPLICATION_JSON, APPLICATION_CBOR];
}

pub(crate) enum ContentType {
    Json,
    Cbor,
}

impl ContentType {
    pub fn extract(headers: &http::HeaderMap) -> Option<Self> {
        let bytes = headers.get(http::header::CONTENT_TYPE)?.as_bytes();
        let bytes = match bytes.iter().position(|&b| b == b';') {
            Some(pos) => &bytes[..pos],
            None => bytes,
        };
        if bytes == content_types::APPLICATION_JSON.as_bytes() {
            Some(ContentType::Json)
        } else if bytes == content_types::APPLICATION_CBOR.as_bytes() {
            Some(ContentType::Cbor)
        } else {
            None
        }
    }

    pub fn supported() -> &'static [http::HeaderValue] {
        &content_types::SUPPORTED
    }
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::application_json()
    }
}

impl ResponseFormat {
    pub fn application_json() -> Self {
        ResponseFormat::Complete(CompleteResponseFormat::Json)
    }

    pub fn supported_media_types() -> &'static [MediaType<'static>] {
        accept::SUPPORTED
    }
}

impl CompleteResponseFormat {
    pub fn to_content_type_header_value(self) -> http::HeaderValue {
        match self {
            CompleteResponseFormat::Json => content_types::APPLICATION_JSON.clone(),
            CompleteResponseFormat::GraphqlResponseJson => content_types::APPLICATION_GRAPHQL_RESPONSE_JSON.clone(),
        }
    }
}

mod accept {
    use mediatype::MediaType;
    use mediatype::names::*;

    pub const STAR_STAR: MediaType<'static> = MediaType::new(_STAR, _STAR);
    pub const APPLICATION_STAR: MediaType<'static> = MediaType::new(APPLICATION, _STAR);
    pub const MULTIPART_MIXED: MediaType<'static> = MediaType::new(MULTIPART, MIXED);
    pub const TEXT_EVENT_STREAM: MediaType<'static> = MediaType::new(TEXT, EVENT_STREAM);
    pub const APPLICATION_JSON: MediaType<'static> = MediaType::new(APPLICATION, JSON);
    pub const APPLICATION_GRAPHQL_RESPONSE_JSON: MediaType<'static> = MediaType::from_parts(
        APPLICATION,
        mediatype::Name::new_unchecked("graphql-response"),
        Some(mediatype::Name::new_unchecked("json")),
        &[],
    );
    pub const SUPPORTED: &[MediaType<'static>] = &[
        STAR_STAR,
        APPLICATION_STAR,
        APPLICATION_JSON,
        APPLICATION_GRAPHQL_RESPONSE_JSON,
        TEXT_EVENT_STREAM,
        MULTIPART_MIXED,
    ];
}

impl ResponseFormat {
    pub fn extract_from(headers: &http::HeaderMap) -> Option<Self> {
        if !headers.contains_key("accept") {
            return Some(ResponseFormat::default());
        }
        let (mediatype, _) = headers
            .get_all("accept")
            .into_iter()
            .filter_map(|value| value.to_str().ok().map(mediatype::MediaTypeList::new))
            .flatten()
            .filter_map(Result::ok)
            .filter(|mediatype| {
                // Get the mediatype without parameters
                accept::SUPPORTED.iter().any(|md| md == &mediatype.essence())
            })
            .map(|mediatype| {
                let quality_value = mediatype
                    .params
                    .iter()
                    .find(|(name, _)| name == "q")
                    .and_then(|(_, value)| value.as_str().parse::<f32>().ok())
                    .unwrap_or(1.0);

                (mediatype, quality_value)
            })
            .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))?;

        let essence = mediatype.essence();
        if essence == accept::STAR_STAR || essence == accept::APPLICATION_STAR {
            Some(ResponseFormat::default())
        } else if essence == accept::APPLICATION_JSON {
            Some(ResponseFormat::Complete(CompleteResponseFormat::Json))
        } else if essence == accept::APPLICATION_GRAPHQL_RESPONSE_JSON {
            Some(ResponseFormat::Complete(CompleteResponseFormat::GraphqlResponseJson))
        } else if essence == accept::MULTIPART_MIXED {
            Some(ResponseFormat::Streaming(StreamingResponseFormat::IncrementalDelivery))
        } else if essence == accept::TEXT_EVENT_STREAM {
            Some(ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverSSE))
        } else {
            None
        }
    }
}
