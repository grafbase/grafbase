use axum::response::IntoResponse;
use http::{HeaderMap, StatusCode};
use serde::{ser::SerializeSeq, Serializer};
use serde_json::Value;

pub enum BatchResponse {
    Single(gateway_v2::Response),
    Batch(Vec<gateway_v2::Response>),
}

impl IntoResponse for BatchResponse {
    fn into_response(self) -> axum::response::Response {
        let serialized_size = self.serialized_size();
        match self {
            BatchResponse::Single(response) => (response.status, response.headers, response.bytes).into_response(),
            BatchResponse::Batch(responses) => {
                // These are awkward af to deal with - they have bytes inside and we need to make
                // them into a JSON list. So we're going to have to do a serde roundtrip.
                // Inefficient, but I don't have time to go refactoring everything rn. It's also
                // unclear that avoiding this would actually be more efficient overall, because the
                // response containing bytes is itself an optimisation - avoids serializing twice
                // when caching and avoids a serde roundtrip when reading from the cache.  Urgh,
                let Ok((headers, bytes)) = headers_and_body(responses, serialized_size) else {
                    todo!("Error")
                };

                (StatusCode::OK, headers, bytes).into_response()
            }
        }
    }
}

impl BatchResponse {
    pub fn serialized_size(&self) -> usize {
        match self {
            BatchResponse::Single(response) => response.bytes.len(),
            BatchResponse::Batch(responses) => {
                // The bytes themselves
                responses.iter().map(|response| response.bytes.len()).sum::<usize>() +
                    // Commas
                    (responses.len() - 1) +
                    // The []s
                    2
            }
        }
    }
}

fn headers_and_body(
    responses: Vec<gateway_v2::Response>,
    serialized_size: usize,
) -> Result<(HeaderMap, Vec<u8>), serde_json::Error> {
    let mut headers = HeaderMap::new();

    let mut bytes = Vec::with_capacity(serialized_size);
    let mut serializer = serde_json::Serializer::new(&mut bytes);
    let mut serializer = serializer.serialize_seq(Some(responses.len()))?;

    for response in responses {
        headers.extend(response.headers);
        serializer.serialize_element(&serde_json::from_slice::<Value>(&response.bytes)?)?;
    }

    serializer.end().unwrap();

    Ok((headers, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialized_size_is_accurate() {
        let response = BatchResponse::Batch(vec![
            gateway_v2::Response::unauthorized(),
            gateway_v2::Response::unauthorized(),
            gateway_v2::Response::unauthorized(),
        ]);
        let serialized_size = response.serialized_size();
        let BatchResponse::Batch(responses) = response else {
            unreachable!()
        };
        let (_, body) = headers_and_body(responses, serialized_size).unwrap();

        assert_eq!(serialized_size, body.len());
    }
}
