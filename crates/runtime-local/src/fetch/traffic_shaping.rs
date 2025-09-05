use std::{future::Future, hash::Hash, sync::Arc};

use bytes::Bytes;
use dashmap::DashMap;
use engine_schema::GraphqlSubgraphId;
use gateway_config::TrafficShapingConfig;
use runtime::fetch::FetchRequest;

use crate::fetch::FetchResponse;

pub struct TrafficShaping {
    config: TrafficShapingConfig,
    inflight: DashMap<&'static RequestKey, InflightRequest, rapidhash::fast::RandomState>,
}

impl TrafficShaping {
    pub fn new(config: &TrafficShapingConfig) -> Self {
        Self {
            config: config.clone(),
            inflight: DashMap::default(),
        }
    }

    pub async fn deduplicate<'a, F>(
        &self,
        request: FetchRequest<'a>,
        f: impl FnOnce(FetchRequest<'a>) -> F,
    ) -> FetchResponse
    where
        F: Future<Output = FetchResponse> + Send,
    {
        if !self.config.inflight_deduplication {
            return f(request).await;
        }
        let request_key = Box::new(RequestKey::from(&request));
        // SAFETY: First, we boxed the key so it won't move. Second there are two cases:
        //         - We add the value in the map, and it's our box that gets stored within the
        //           entry. In that case the key will only be used as long as the value is present.
        //         - The entry exists, our reference will simply disappear.
        let key = unsafe { std::mem::transmute::<&RequestKey, &'static RequestKey>(request_key.as_ref()) };
        let cell = self
            .inflight
            .entry(key)
            .or_insert_with(|| InflightRequest {
                request_key,
                cell: Arc::new(tokio::sync::OnceCell::new()),
            })
            .cell
            .clone();

        cell.get_or_init(|| async {
            let result = f(request).await;
            self.inflight.remove(key);
            result
        })
        .await;

        match Arc::try_unwrap(cell) {
            Ok(cell) => cell.into_inner().unwrap(),
            Err(cell) => cell.get().unwrap().clone(),
        }
    }
}

struct InflightRequest {
    #[allow(unused)] // Used to keep the key in the dashmap alive.
    request_key: Box<RequestKey>,
    cell: Arc<tokio::sync::OnceCell<FetchResponse>>,
}

#[derive(Debug)]
struct RequestKey {
    subgraph_id: GraphqlSubgraphId,
    header_sections: Vec<(u32, u32)>,
    method_start: u32,
    url_start: u32,
    parts_bytes: Vec<u8>,
    body: Bytes,
}

impl PartialEq for RequestKey {
    fn eq(&self, other: &Self) -> bool {
        // Compare Subgraph and other
        // cheap comparisons first
        if !(self.subgraph_id == other.subgraph_id
            && self.body.len() == other.body.len()
            && self.parts_bytes.len() == other.parts_bytes.len()
            && self.header_sections.len() == other.header_sections.len()
            && self.method_start == other.method_start
            && self.url_start == other.url_start)
        {
            return false;
        }

        // Compare Method
        if self.parts_bytes[self.method_start as usize..] != other.parts_bytes[other.method_start as usize..] {
            return false;
        }

        // Compare URL
        if self.parts_bytes[self.url_start as usize..self.method_start as usize]
            != other.parts_bytes[other.url_start as usize..self.method_start as usize]
        {
            return false;
        }

        // Compare headers
        for ((self_start, self_end), (other_start, other_end)) in self
            .header_sections
            .iter()
            .copied()
            .zip(other.header_sections.iter().copied())
        {
            if self.parts_bytes[self_start as usize..self_end as usize]
                != other.parts_bytes[other_start as usize..other_end as usize]
            {
                return false;
            }
        }

        // Compare body
        self.body == other.body
    }
}

impl Eq for RequestKey {}

impl Hash for RequestKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.subgraph_id.hash(state);
        self.parts_bytes[self.method_start as usize..].hash(state);
        self.parts_bytes[self.url_start as usize..self.method_start as usize].hash(state);
        for (start, end) in self.header_sections.iter().copied() {
            self.parts_bytes[(start as usize)..(end as usize)].hash(state);
        }
        self.body.hash(state);
    }
}

impl From<&FetchRequest<'_>> for RequestKey {
    fn from(req: &FetchRequest<'_>) -> Self {
        let subgraph_id = req.subgraph_id;
        let mut parts_bytes = Vec::with_capacity(req.url.as_str().len() + req.headers.len() * 20);
        let mut header_sections = Vec::with_capacity(req.headers.keys_len());

        for (name, value) in req.headers.iter() {
            let start = parts_bytes.len() as u32;
            parts_bytes.extend_from_slice(name.as_str().as_bytes());
            parts_bytes.extend_from_slice(value.as_bytes());
            let end = parts_bytes.len() as u32;
            header_sections.push((start, end));
        }

        header_sections.sort_unstable_by(|&(left_start, left_end), &(right_start, right_end)| {
            parts_bytes[left_start as usize..left_end as usize]
                .cmp(&parts_bytes[right_start as usize..right_end as usize])
        });

        let url_start = parts_bytes.len() as u32;
        parts_bytes.extend_from_slice(req.url.as_str().as_bytes());

        let method_start = parts_bytes.len() as u32;
        parts_bytes.extend_from_slice(req.method.as_str().as_bytes());

        RequestKey {
            subgraph_id,
            header_sections,
            method_start,
            url_start,
            parts_bytes,
            body: req.body.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use std::time::Duration;

    fn create_request_key(
        subgraph_id: u16,
        method: http::Method,
        headers: Vec<(&str, &str)>,
        url: &str,
        body: &[u8],
    ) -> RequestKey {
        (&FetchRequest {
            subgraph_id: subgraph_id.into(),
            method,
            headers: {
                let mut map = http::HeaderMap::new();
                for (name, value) in &headers {
                    map.append(
                        http::header::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                        http::header::HeaderValue::from_str(value).unwrap(),
                    );
                }
                map
            },
            url: Cow::Owned(url.parse().unwrap()),
            body: Bytes::copy_from_slice(body),
            timeout: Duration::from_secs(30),
        })
            .into()
    }

    fn hash_value(key: &RequestKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn identical_requests_are_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json"), ("Authorization", "Bearer token")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json"), ("Authorization", "Bearer token")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn different_subgraph_ids_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            2,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn different_urls_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api/v2",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn different_bodies_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"body 1",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"body 2",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn headers_order_does_not_matter() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Authorization", "Bearer token"), ("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json"), ("Authorization", "Bearer token")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn different_header_values_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "text/plain")],
            "https://example.com/api",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn different_header_names_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Accept", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn additional_headers_make_keys_different() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json"), ("X-Custom", "value")],
            "https://example.com/api",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn empty_headers_are_equal() {
        let key1 = create_request_key(1, http::Method::POST, vec![], "https://example.com/api", b"test body");

        let key2 = create_request_key(1, http::Method::POST, vec![], "https://example.com/api", b"test body");

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn empty_bodies_are_equal() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn header_names_are_case_insensitive() {
        // HTTP header names are case-insensitive per RFC 7230
        // The http::HeaderName type normalizes them to lowercase
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("content-type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        // Headers should be equal since names are normalized to lowercase
        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn header_values_are_case_sensitive() {
        // While header names are case-insensitive, header VALUES are case-sensitive
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("Authorization", "bearer token")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Authorization", "Bearer token")],
            "https://example.com/api",
            b"test body",
        );

        // Values differ in case, so keys should be different
        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn case_sensitive_urls() {
        let key1 = create_request_key(1, http::Method::POST, vec![], "https://example.com/API", b"test body");

        let key2 = create_request_key(1, http::Method::POST, vec![], "https://example.com/api", b"test body");

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn multiple_headers_with_same_sorted_order() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("A-Header", "value1"), ("B-Header", "value2"), ("C-Header", "value3")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("B-Header", "value2"), ("C-Header", "value3"), ("A-Header", "value1")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn header_with_special_characters() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("X-Special", "value with spaces, symbols: !@#$%^&*()=+[]{}|;':\"<>?,./")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("X-Special", "value with spaces, symbols: !@#$%^&*()=+[]{}|;':\"<>?,./")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn url_with_query_params() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![],
            "https://example.com/api?param1=value1&param2=value2",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![],
            "https://example.com/api?param2=value2&param1=value1",
            b"test body",
        );

        // Query parameter order matters in URL
        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn large_body_comparison() {
        let large_body1 = vec![0u8; 10000];
        let mut large_body2 = vec![0u8; 10000];
        large_body2[9999] = 1; // Only last byte different

        let key1 = create_request_key(1, http::Method::POST, vec![], "https://example.com/api", &large_body1);

        let key2 = create_request_key(1, http::Method::POST, vec![], "https://example.com/api", &large_body2);

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn hash_consistency() {
        let key = create_request_key(
            42,
            http::Method::POST,
            vec![("Authorization", "Bearer abc123"), ("Content-Type", "application/json")],
            "https://api.example.com/graphql",
            b"{\"query\": \"{ user { id name } }\"}",
        );

        // Hash should be consistent across multiple calls
        let hash1 = hash_value(&key);
        let hash2 = hash_value(&key);
        let hash3 = hash_value(&key);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn unicode_in_headers_and_url() {
        let key1 = create_request_key(
            1,
            http::Method::POST,
            vec![("X-Unicode", "Hello 世界 🚀")],
            "https://example.com/api/路径",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("X-Unicode", "Hello 世界 🚀")],
            "https://example.com/api/路径",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn eq_reflexivity() {
        let key = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key, key);
    }

    #[test]
    fn different_methods_are_not_equal() {
        let key1 = create_request_key(
            1,
            http::Method::GET,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::POST,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn same_methods_are_equal() {
        let key1 = create_request_key(
            1,
            http::Method::PUT,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            http::Method::PUT,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn all_common_methods_differ() {
        let methods = vec![
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::PATCH,
            http::Method::HEAD,
            http::Method::OPTIONS,
        ];

        let mut keys = Vec::new();
        for method in &methods {
            keys.push(create_request_key(
                1,
                method.clone(),
                vec![("Content-Type", "application/json")],
                "https://example.com/api",
                b"test body",
            ));
        }

        // Each method should produce a different key
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(
                    keys[i], keys[j],
                    "Methods {:?} and {:?} should not be equal",
                    methods[i], methods[j]
                );
                assert_ne!(
                    hash_value(&keys[i]),
                    hash_value(&keys[j]),
                    "Methods {:?} and {:?} should have different hashes",
                    methods[i],
                    methods[j]
                );
            }
        }
    }

    #[test]
    fn custom_method_equality() {
        let custom_method = http::Method::from_bytes(b"CUSTOM").unwrap();

        let key1 = create_request_key(
            1,
            custom_method.clone(),
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            custom_method,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        assert_eq!(key1, key2);
        assert_eq!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn method_case_matters() {
        // HTTP methods are case-sensitive according to RFC
        let method1 = http::Method::from_bytes(b"Get").unwrap();
        let method2 = http::Method::GET;

        let key1 = create_request_key(
            1,
            method1,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        let key2 = create_request_key(
            1,
            method2,
            vec![("Content-Type", "application/json")],
            "https://example.com/api",
            b"test body",
        );

        // "Get" and "GET" are different methods
        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn method_only_difference() {
        // Test that method alone can differentiate keys when everything else is identical
        let key_get = create_request_key(1, http::Method::GET, vec![], "https://example.com", b"");

        let key_post = create_request_key(1, http::Method::POST, vec![], "https://example.com", b"");

        let key_put = create_request_key(1, http::Method::PUT, vec![], "https://example.com", b"");

        assert_ne!(key_get, key_post);
        assert_ne!(key_get, key_put);
        assert_ne!(key_post, key_put);

        assert_ne!(hash_value(&key_get), hash_value(&key_post));
        assert_ne!(hash_value(&key_get), hash_value(&key_put));
        assert_ne!(hash_value(&key_post), hash_value(&key_put));
    }

    #[test]
    fn method_with_complex_request() {
        // Test methods with more complex requests
        let key1 = create_request_key(
            1,
            http::Method::PATCH,
            vec![
                ("Authorization", "Bearer token123"),
                ("Content-Type", "application/json"),
                ("X-Request-Id", "abc-123"),
            ],
            "https://api.example.com/users/123?include=profile&expand=permissions",
            b"{\"name\": \"John Doe\", \"email\": \"john@example.com\"}",
        );

        let key2 = create_request_key(
            1,
            http::Method::PUT,
            vec![
                ("Authorization", "Bearer token123"),
                ("Content-Type", "application/json"),
                ("X-Request-Id", "abc-123"),
            ],
            "https://api.example.com/users/123?include=profile&expand=permissions",
            b"{\"name\": \"John Doe\", \"email\": \"john@example.com\"}",
        );

        // Only method differs
        assert_ne!(key1, key2);
        assert_ne!(hash_value(&key1), hash_value(&key2));
    }

    #[test]
    fn method_hash_distribution() {
        // Ensure different methods produce well-distributed hashes
        let methods = vec![
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::PATCH,
        ];

        let mut hashes = Vec::new();
        for method in methods {
            let key = create_request_key(1, method, vec![], "https://example.com", b"");
            hashes.push(hash_value(&key));
        }

        // All hashes should be unique
        let unique_hashes: std::collections::HashSet<_> = hashes.iter().collect();
        assert_eq!(hashes.len(), unique_hashes.len(), "All method hashes should be unique");
    }
}
