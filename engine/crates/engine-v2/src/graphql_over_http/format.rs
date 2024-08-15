use mediatype::MediaType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResponseFormat {
    Complete(CompleteResponseFormat),
    Streaming(StreamingResponseFormat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompleteResponseFormat {
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
pub(crate) enum StreamingResponseFormat {
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
    pub static APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");
    pub static APPLICATION_GRAPHQL_RESPONSE_JSON: http::HeaderValue =
        http::HeaderValue::from_static("application/graphql-response+json");
}

impl ResponseFormat {
    pub fn application_json() -> Self {
        ResponseFormat::Complete(CompleteResponseFormat::Json)
    }

    pub fn supported_media_types() -> &'static [MediaType<'static>] {
        mediatypes::SUPPORTED
    }
}

impl CompleteResponseFormat {
    pub fn to_content_type(self) -> http::HeaderValue {
        match self {
            CompleteResponseFormat::Json => content_types::APPLICATION_JSON.clone(),
            CompleteResponseFormat::GraphqlResponseJson => content_types::APPLICATION_GRAPHQL_RESPONSE_JSON.clone(),
        }
    }
}

mod mediatypes {
    use mediatype::MediaType;

    pub const MULTIPART_MIXED: MediaType<'static> =
        MediaType::new(mediatype::names::MULTIPART, mediatype::names::MIXED);
    pub const TEXT_EVENT_STREAM: MediaType<'static> =
        MediaType::new(mediatype::names::TEXT, mediatype::names::EVENT_STREAM);
    pub const APPLICATION_JSON: MediaType<'static> =
        MediaType::new(mediatype::names::APPLICATION, mediatype::names::JSON);
    pub const APPLICATION_GRAPHQL_RESPONSE_JSON: MediaType<'static> = MediaType::from_parts(
        mediatype::names::APPLICATION,
        mediatype::Name::new_unchecked("graphql-response"),
        Some(mediatype::Name::new_unchecked("json")),
        &[],
    );
    pub const SUPPORTED: &[MediaType<'static>] = &[
        APPLICATION_JSON,
        APPLICATION_GRAPHQL_RESPONSE_JSON,
        TEXT_EVENT_STREAM,
        MULTIPART_MIXED,
    ];
}

impl ResponseFormat {
    pub fn extract_from(headers: &http::HeaderMap) -> Option<Self> {
        headers
            .get_all("accept")
            .into_iter()
            .filter_map(|value| value.to_str().ok().and_then(Self::from_header))
            .last()
    }

    fn from_header(value: &str) -> Option<Self> {
        let (mediatype, _) = mediatype::MediaTypeList::new(value)
            .filter_map(Result::ok)
            .filter(|mediatype| {
                // Get the mediatype without parameters
                mediatypes::SUPPORTED.iter().any(|md| md == &mediatype.essence())
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
        if essence == mediatypes::APPLICATION_JSON {
            Some(ResponseFormat::Complete(CompleteResponseFormat::Json))
        } else if essence == mediatypes::APPLICATION_GRAPHQL_RESPONSE_JSON {
            Some(ResponseFormat::Complete(CompleteResponseFormat::GraphqlResponseJson))
        } else if essence == mediatypes::MULTIPART_MIXED {
            Some(ResponseFormat::Streaming(StreamingResponseFormat::IncrementalDelivery))
        } else if essence == mediatypes::TEXT_EVENT_STREAM {
            Some(ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverSSE))
        } else {
            None
        }
    }
}