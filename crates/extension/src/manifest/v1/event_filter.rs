use enumflags2::{BitFlags, bitflags};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventFilter {
    All,
    Types(BitFlags<EventFilterType>),
}

impl serde::Serialize for EventFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializing::serialize(self, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for EventFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serializing::deserialize(deserializer)
    }
}

#[bitflags]
#[repr(u16)]
#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventFilterType {
    #[serde(rename = "operation")]
    Operation = 1 << 0,
    #[serde(rename = "subgraph_request")]
    SubgraphRequest = 1 << 1,
    #[serde(rename = "http_request")]
    HttpRequest = 1 << 2,
    #[serde(rename = "extension")]
    Extension = 1 << 3,
}

pub(super) mod serializing {
    use super::*;
    use enumflags2::BitFlags;
    use serde::de::{self, SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserializer, Serialize, Serializer};
    use std::fmt;

    pub fn serialize<S>(filter: &EventFilter, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let EventFilter::Types(types) = filter else {
            return "*".serialize(serializer);
        };

        let mut vec = Vec::new();

        // Add each permission to the vector if it's set
        if types.contains(EventFilterType::Operation) {
            vec.push("operation");
        }

        if types.contains(EventFilterType::SubgraphRequest) {
            vec.push("subgraph_request");
        }

        if types.contains(EventFilterType::HttpRequest) {
            vec.push("http_request");
        }

        if types.contains(EventFilterType::Extension) {
            vec.push("extension");
        }

        let mut seq = serializer.serialize_seq(Some(vec.len()))?;

        for item in vec {
            seq.serialize_element(item)?;
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EventFilter, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventTypeVisitor;

        impl<'de> Visitor<'de> for EventTypeVisitor {
            type Value = EventFilter;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence of event type strings, or *")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut types = BitFlags::empty();

                while let Some(value) = seq.next_element::<String>()? {
                    match value.as_str() {
                        "operation" => types |= EventFilterType::Operation,
                        "subgraph_request" => types |= EventFilterType::SubgraphRequest,
                        "http_request" => types |= EventFilterType::HttpRequest,
                        "extension" => types |= EventFilterType::Extension,
                        _ => {
                            return Err(de::Error::unknown_variant(
                                &value,
                                &["operation", "subgraph_request", "http_request", "extension"],
                            ));
                        }
                    }
                }

                Ok(EventFilter::Types(types))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "*" => Ok(EventFilter::All),
                    _ => Err(de::Error::unknown_variant(
                        value,
                        &["*", "operation", "subgraph_request", "http_request", "extension"],
                    )),
                }
            }
        }

        deserializer.deserialize_any(EventTypeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestWrapper {
        #[serde(with = "super::serializing")]
        filter: EventFilter,
    }

    #[test]
    fn test_serialize_all() {
        let wrapper = TestWrapper {
            filter: EventFilter::All,
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#"{"filter":"*"}"#);
    }

    #[test]
    fn test_deserialize_all() {
        let json = r#"{"filter":"*"}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        assert_eq!(EventFilter::All, wrapper.filter);
    }

    #[test]
    fn test_serialize_empty_types() {
        let wrapper = TestWrapper {
            filter: EventFilter::Types(BitFlags::empty()),
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#"{"filter":[]}"#);
    }

    #[test]
    fn test_deserialize_empty_types() {
        let json = r#"{"filter":[]}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        assert_eq!(EventFilter::Types(BitFlags::empty()), wrapper.filter);
    }

    #[test]
    fn test_serialize_single_type() {
        let wrapper = TestWrapper {
            filter: EventFilter::Types(EventFilterType::Operation.into()),
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#"{"filter":["operation"]}"#);
    }

    #[test]
    fn test_deserialize_single_type() {
        let json = r#"{"filter":["operation"]}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        let flags = EventFilterType::Operation;
        assert_eq!(EventFilter::Types(flags.into()), wrapper.filter);
    }

    #[test]
    fn test_serialize_multiple_types() {
        let wrapper = TestWrapper {
            filter: EventFilter::Types(
                EventFilterType::Operation | EventFilterType::HttpRequest | EventFilterType::Extension,
            ),
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#"{"filter":["operation","http_request","extension"]}"#);
    }

    #[test]
    fn test_deserialize_multiple_types() {
        let json = r#"{"filter":["subgraph_request","http_request"]}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        let flags = EventFilterType::SubgraphRequest | EventFilterType::HttpRequest;
        assert_eq!(EventFilter::Types(flags), wrapper.filter);
    }

    #[test]
    fn test_serialize_all_types() {
        let wrapper = TestWrapper {
            filter: EventFilter::Types(
                EventFilterType::Operation
                    | EventFilterType::SubgraphRequest
                    | EventFilterType::HttpRequest
                    | EventFilterType::Extension,
            ),
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(
            json,
            r#"{"filter":["operation","subgraph_request","http_request","extension"]}"#
        );
    }

    #[test]
    fn test_deserialize_all_types() {
        let json = r#"{"filter":["operation","subgraph_request","http_request","extension"]}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        assert_eq!(EventFilter::Types(BitFlags::all()), wrapper.filter);
    }

    #[test]
    fn test_deserialize_unknown_type() {
        let json = r#"{"filter":["unknown_type"]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown variant"));
    }

    #[test]
    fn test_deserialize_mixed_valid_invalid() {
        let json = r#"{"filter":["operation","invalid","http_request"]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_duplicate_types() {
        // Should handle duplicates gracefully
        let json = r#"{"filter":["operation","operation","http_request","operation"]}"#;
        let wrapper: TestWrapper = serde_json::from_str(json).unwrap();

        let flags = EventFilterType::Operation | EventFilterType::HttpRequest;
        assert_eq!(EventFilter::Types(flags), wrapper.filter);
    }

    #[test]
    fn test_deserialize_wrong_type() {
        // Test that numbers are rejected
        let json = r#"{"filter":123}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip_all() {
        let original = TestWrapper {
            filter: EventFilter::All,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TestWrapper = serde_json::from_str(&json).unwrap();

        assert_eq!(original.filter, deserialized.filter);
    }

    #[test]
    fn test_round_trip_various_combinations() {
        let test_cases = vec![
            EventFilter::Types(BitFlags::empty()),
            EventFilter::Types(EventFilterType::Operation.into()),
            EventFilter::Types(EventFilterType::SubgraphRequest.into()),
            EventFilter::Types(EventFilterType::HttpRequest.into()),
            EventFilter::Types(EventFilterType::Extension.into()),
            EventFilter::Types(EventFilterType::Operation | EventFilterType::SubgraphRequest),
            EventFilter::Types(EventFilterType::HttpRequest | EventFilterType::Extension),
            EventFilter::Types(
                EventFilterType::Operation | EventFilterType::SubgraphRequest | EventFilterType::HttpRequest,
            ),
            EventFilter::Types(
                EventFilterType::Operation
                    | EventFilterType::SubgraphRequest
                    | EventFilterType::HttpRequest
                    | EventFilterType::Extension,
            ),
        ];

        for filter in test_cases {
            let original = TestWrapper { filter };
            let json = serde_json::to_string(&original).unwrap();
            let deserialized: TestWrapper = serde_json::from_str(&json).unwrap();

            assert_eq!(original.filter, deserialized.filter)
        }
    }

    #[test]
    fn test_order_independence() {
        // Test that order doesn't matter in deserialization
        let json1 = r#"{"filter":["operation","http_request","extension"]}"#;
        let json2 = r#"{"filter":["extension","operation","http_request"]}"#;
        let json3 = r#"{"filter":["http_request","extension","operation"]}"#;

        let wrapper1: TestWrapper = serde_json::from_str(json1).unwrap();
        let wrapper2: TestWrapper = serde_json::from_str(json2).unwrap();
        let wrapper3: TestWrapper = serde_json::from_str(json3).unwrap();

        match (&wrapper1.filter, &wrapper2.filter, &wrapper3.filter) {
            (EventFilter::Types(t1), EventFilter::Types(t2), EventFilter::Types(t3)) => {
                assert_eq!(t1, t2);
                assert_eq!(t2, t3);
            }
            _ => unreachable!("Expected EventFilter::Types"),
        }
    }

    #[test]
    fn test_deserialize_null() {
        // Test that null is rejected
        let json = r#"{"filter":null}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_empty_string() {
        // Test that empty string is rejected
        let json = r#"{"filter":""}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown variant"));
    }

    #[test]
    fn test_deserialize_case_sensitivity() {
        // Test that type names are case sensitive
        let json = r#"{"filter":["Operation","SUBGRAPH_REQUEST"]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_whitespace_in_types() {
        // Test that whitespace in type names is not allowed
        let json = r#"{"filter":["operation ","  http_request"]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_nested_arrays() {
        // Test that nested arrays are rejected
        let json = r#"{"filter":[["operation"]]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_object_type() {
        // Test that objects are rejected
        let json = r#"{"filter":{"type":"operation"}}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_order_consistency() {
        // Test that serialization always produces the same order
        let filter =
            EventFilter::Types(EventFilterType::Extension | EventFilterType::Operation | EventFilterType::HttpRequest);

        let wrapper = TestWrapper { filter };
        let json1 = serde_json::to_string(&wrapper).unwrap();
        let json2 = serde_json::to_string(&wrapper).unwrap();

        assert_eq!(json1, json2);
        // Verify specific order (based on the implementation)
        assert_eq!(json1, r#"{"filter":["operation","http_request","extension"]}"#);
    }

    #[test]
    fn test_deserialize_boolean() {
        // Test that boolean values are rejected
        let json = r#"{"filter":true}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_mixed_types_array() {
        // Test that arrays with mixed types are rejected
        let json = r#"{"filter":["operation", 123, true]}"#;
        let result: Result<TestWrapper, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_wrapper_field() {
        // Test serialization/deserialization when EventFilter is used in Option
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct OptionalWrapper {
            filter: Option<EventFilter>,
        }

        let wrapper = OptionalWrapper { filter: None };
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#"{"filter":null}"#);

        let deserialized: OptionalWrapper = serde_json::from_str(&json).unwrap();
        assert_eq!(wrapper, deserialized);
    }

    #[test]
    fn test_manifest_integration() {
        // Test EventFilter integration with a Manifest-like structure
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct ManifestLike {
            name: String,
            version: String,
            event_filter: Option<EventFilter>,
            other_field: bool,
        }

        // Test with EventFilter::All
        let manifest1 = ManifestLike {
            name: "test-extension".to_string(),
            version: "1.0.0".to_string(),
            event_filter: Some(EventFilter::All),
            other_field: true,
        };

        let json1 = serde_json::to_string(&manifest1).unwrap();
        assert!(json1.contains(r#""event_filter":"*""#));
        let deserialized1: ManifestLike = serde_json::from_str(&json1).unwrap();
        assert_eq!(manifest1, deserialized1);

        // Test with specific types
        let manifest2 = ManifestLike {
            name: "test-extension".to_string(),
            version: "1.0.0".to_string(),
            event_filter: Some(EventFilter::Types(
                EventFilterType::Operation | EventFilterType::HttpRequest,
            )),
            other_field: false,
        };

        let json2 = serde_json::to_string(&manifest2).unwrap();
        assert!(json2.contains(r#""event_filter":["operation","http_request"]"#));
        let deserialized2: ManifestLike = serde_json::from_str(&json2).unwrap();
        assert_eq!(manifest2, deserialized2);

        // Test with None
        let manifest3 = ManifestLike {
            name: "test-extension".to_string(),
            version: "1.0.0".to_string(),
            event_filter: None,
            other_field: true,
        };

        let json3 = serde_json::to_string(&manifest3).unwrap();
        assert!(json3.contains(r#""event_filter":null"#));
        let deserialized3: ManifestLike = serde_json::from_str(&json3).unwrap();
        assert_eq!(manifest3, deserialized3);

        // Test deserializing from JSON with event_filter field
        let json_with_filter = r#"{
            "name": "my-extension",
            "version": "2.0.0",
            "event_filter": ["subgraph_request", "extension"],
            "other_field": false
        }"#;

        let manifest4: ManifestLike = serde_json::from_str(json_with_filter).unwrap();
        assert_eq!(manifest4.name, "my-extension");

        let flags = EventFilterType::SubgraphRequest | EventFilterType::Extension;
        assert_eq!(EventFilter::Types(flags), manifest4.event_filter.unwrap());
    }
}
