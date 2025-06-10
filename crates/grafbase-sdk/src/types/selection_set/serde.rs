use crate::wit::selection_set_resolver_types as wit;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

impl Serialize for wit::Field {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(4)?;
        tuple.serialize_element(&self.alias)?;
        tuple.serialize_element(&self.definition_id)?;
        tuple.serialize_element(&self.arguments)?;
        tuple.serialize_element(&self.selection_set)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for wit::Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldVisitor {
            type Value = wit::Field;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a tuple of 4 elements")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<wit::Field, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let alias = seq.next_element()?.unwrap_or(None);
                let definition_id = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let arguments = seq.next_element()?.unwrap_or(None);
                let selection_set = seq.next_element()?.unwrap_or(None);

                Ok(wit::Field {
                    alias,
                    definition_id,
                    arguments,
                    selection_set,
                })
            }
        }

        deserializer.deserialize_tuple(4, FieldVisitor)
    }
}

impl Serialize for wit::SelectionSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.requires_typename)?;
        tuple.serialize_element(&self.fields_ordered_by_parent_entity)?;
        tuple.end()
    }
}

impl<'de> Deserialize<'de> for wit::SelectionSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SelectionSetVisitor;

        impl<'de> serde::de::Visitor<'de> for SelectionSetVisitor {
            type Value = wit::SelectionSet;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a tuple of 2 elements")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<wit::SelectionSet, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let requires_typename = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let fields_ordered_by_parent_entity = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                Ok(wit::SelectionSet {
                    requires_typename,
                    fields_ordered_by_parent_entity,
                })
            }
        }

        deserializer.deserialize_tuple(2, SelectionSetVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    // Helper function to create a test Field
    fn create_test_field() -> wit::Field {
        wit::Field {
            alias: Some("test_alias".to_string()),
            definition_id: 42,
            arguments: Some(123u16), // arguments is a u16
            selection_set: Some(wit::SelectionSet {
                requires_typename: true,
                fields_ordered_by_parent_entity: (0, 5),
            }),
        }
    }

    // Helper function to create a test Field with None values
    fn create_minimal_field() -> wit::Field {
        wit::Field {
            alias: None,
            definition_id: 123,
            arguments: None,
            selection_set: None,
        }
    }

    // Helper function to create a test SelectionSet
    fn create_test_selection_set() -> wit::SelectionSet {
        wit::SelectionSet {
            requires_typename: false,
            fields_ordered_by_parent_entity: (10, 20),
        }
    }

    #[test]
    fn test_field_serialize() {
        let field = create_test_field();
        let serialized = serde_json::to_string(&field).expect("Failed to serialize Field");

        let expected = r#"["test_alias",42,123,[true,[0,5]]]"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_field_serialize_minimal() {
        let field = create_minimal_field();
        let serialized = serde_json::to_string(&field).expect("Failed to serialize minimal Field");

        let expected = r#"[null,123,null,null]"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_field_deserialize() {
        let json = r#"["test_alias",42,123,[true,[0,5]]]"#;
        let field: wit::Field = serde_json::from_str(json).expect("Failed to deserialize Field");

        assert_eq!(field.alias, Some("test_alias".to_string()));
        assert_eq!(field.definition_id, 42);
        assert_eq!(field.arguments, Some(123u16));

        let selection_set = field.selection_set.expect("SelectionSet should be Some");
        assert!(selection_set.requires_typename);
        assert_eq!(selection_set.fields_ordered_by_parent_entity, (0, 5));
    }

    #[test]
    fn test_field_deserialize_minimal() {
        let json = r#"[null,123,null,null]"#;
        let field: wit::Field = serde_json::from_str(json).expect("Failed to deserialize minimal Field");

        assert_eq!(field.alias, None);
        assert_eq!(field.definition_id, 123);
        assert_eq!(field.arguments, None);
        // Note: We can't use assert_eq! with SelectionSet due to missing PartialEq
        assert!(field.selection_set.is_none());
    }

    #[test]
    fn test_field_deserialize_with_null_values() {
        let json = r#"[null,456,null,null]"#;
        let field: wit::Field = serde_json::from_str(json).expect("Failed to deserialize Field with nulls");

        assert_eq!(field.alias, None);
        assert_eq!(field.definition_id, 456);
        assert_eq!(field.arguments, None);
        assert!(field.selection_set.is_none());
    }

    #[test]
    fn test_field_round_trip() {
        let original = create_test_field();
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: wit::Field = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(original.alias, deserialized.alias);
        assert_eq!(original.definition_id, deserialized.definition_id);
        assert_eq!(original.arguments, deserialized.arguments);

        // Compare SelectionSet fields manually since it doesn't implement PartialEq
        match (original.selection_set, deserialized.selection_set) {
            (Some(orig_ss), Some(deser_ss)) => {
                assert_eq!(orig_ss.requires_typename, deser_ss.requires_typename);
                assert_eq!(
                    orig_ss.fields_ordered_by_parent_entity,
                    deser_ss.fields_ordered_by_parent_entity
                );
            }
            (None, None) => {}
            _ => unreachable!("SelectionSet mismatch"),
        }
    }

    #[test]
    fn test_field_deserialize_invalid_length() {
        let json = r#"["test"]"#; // Too few elements
        let result: Result<wit::Field, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid length"));
    }

    #[test]
    fn test_field_deserialize_wrong_type() {
        let json = r#"["test","not_a_number",null,null]"#; // definition_id should be a number
        let result: Result<wit::Field, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_selection_set_serialize() {
        let selection_set = create_test_selection_set();
        let serialized = serde_json::to_string(&selection_set).expect("Failed to serialize SelectionSet");

        let expected = r#"[false,[10,20]]"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_selection_set_deserialize() {
        let json = r#"[true,[5,15]]"#;
        let selection_set: wit::SelectionSet = serde_json::from_str(json).expect("Failed to deserialize SelectionSet");

        assert!(selection_set.requires_typename);
        assert_eq!(selection_set.fields_ordered_by_parent_entity, (5, 15));
    }

    #[test]
    fn test_selection_set_round_trip() {
        let original = create_test_selection_set();
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: wit::SelectionSet = serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(original.requires_typename, deserialized.requires_typename);
        assert_eq!(
            original.fields_ordered_by_parent_entity,
            deserialized.fields_ordered_by_parent_entity
        );
    }

    #[test]
    fn test_selection_set_deserialize_invalid_length() {
        let json = r#"[true]"#; // Missing fields_ordered_by_parent_entity
        let result: Result<wit::SelectionSet, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid length"));

        let json = r#"[]"#; // Missing both fields
        let result: Result<wit::SelectionSet, _> = serde_json::from_str(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid length"));
    }

    #[test]
    fn test_selection_set_deserialize_wrong_type() {
        let json = r#"["not_a_bool",[0,5]]"#; // requires_typename should be a bool
        let result: Result<wit::SelectionSet, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_field_with_selection_set() {
        let json = r#"["nested_field",999,456,[true,[1,3]]]"#;

        let field: wit::Field = serde_json::from_str(json).expect("Failed to deserialize nested Field");

        assert_eq!(field.alias, Some("nested_field".to_string()));
        assert_eq!(field.definition_id, 999);
        assert_eq!(field.arguments, Some(456u16));

        let selection_set = field.selection_set.expect("SelectionSet should be Some");
        assert!(selection_set.requires_typename);
        assert_eq!(selection_set.fields_ordered_by_parent_entity, (1, 3));
    }
}
