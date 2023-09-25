use mediatype::{MediaType, MediaTypeList, Name};

/// The format ExecutionEngine::execute_stream should return
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamingFormat {
    /// Follow the [incremental delivery spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/IncrementalDelivery.md
    IncrementalDelivery,
    /// Follow the [GraphQL over SSE spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverSSE.md
    GraphQLOverSSE,
}

const INCREMENTAL_MEDIA_TYPE: MediaType<'static> =
    MediaType::new(Name::new_unchecked("multipart"), Name::new_unchecked("mixed"));
const SSE_MEDIA_TYPE: MediaType<'static> =
    MediaType::new(Name::new_unchecked("text"), Name::new_unchecked("event-stream"));

impl StreamingFormat {
    pub fn from_accept_header(header: &str) -> Option<Self> {
        if header.contains("application/graphql-response+json") {
            // Temporarily default to graphql JSON if its present in the headers
            return None;
        }
        let (mediatype, _) = MediaTypeList::new(header)
            .filter_map(Result::ok)
            .filter(|mediatype| {
                // Get the mediatype without parameters
                let essence = mediatype.essence();

                essence == INCREMENTAL_MEDIA_TYPE || essence == SSE_MEDIA_TYPE
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

        let mediatype = mediatype.essence();

        if mediatype == INCREMENTAL_MEDIA_TYPE {
            Some(Self::IncrementalDelivery)
        } else if mediatype == SSE_MEDIA_TYPE {
            Some(Self::GraphQLOverSSE)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_header_parsing() {
        assert_eq!(
            StreamingFormat::from_accept_header("multipart/mixed"),
            Some(StreamingFormat::IncrementalDelivery)
        );
        assert_eq!(
            StreamingFormat::from_accept_header("multipart/mixed,application/json;q=0.9"),
            Some(StreamingFormat::IncrementalDelivery)
        );
        assert_eq!(
            StreamingFormat::from_accept_header("*/*,multipart/mixed;q=0.9,application/json;q=0.8"),
            Some(StreamingFormat::IncrementalDelivery)
        );
        assert_eq!(
            StreamingFormat::from_accept_header(
                "*/*,multipart/mixed;q=0.9,text/event-stream=0.8;application/json;q=0.8"
            ),
            Some(StreamingFormat::IncrementalDelivery)
        );

        assert_eq!(
            StreamingFormat::from_accept_header("text/event-stream"),
            Some(StreamingFormat::GraphQLOverSSE)
        );
        assert_eq!(
            StreamingFormat::from_accept_header("text/event-stream,application/json;q=0.9"),
            Some(StreamingFormat::GraphQLOverSSE)
        );
        assert_eq!(
            StreamingFormat::from_accept_header("*/*,text/event-stream;q=0.9,application/json;q=0.8"),
            Some(StreamingFormat::GraphQLOverSSE)
        );
        assert_eq!(
            StreamingFormat::from_accept_header(
                "*/*,text/event-stream;q=0.9,multipart/mixed=0.8;application/json;q=0.8"
            ),
            Some(StreamingFormat::GraphQLOverSSE)
        );

        assert_eq!(
            StreamingFormat::from_accept_header("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
            None
        );
        assert_eq!(StreamingFormat::from_accept_header("application/json"), None);
        assert_eq!(StreamingFormat::from_accept_header("*/*"), None);
        assert_eq!(
            StreamingFormat::from_accept_header("application/graphql-response+json"),
            None
        );
    }
}
