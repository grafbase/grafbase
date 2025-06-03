use cynic_parser_deser::{ConstDeserializer as _, ValueDeserialize};

/// The composite spec `@is` directive, but in the context of a federated schema.
///
/// `directive @is(field: FieldSelectionMap!) on ARGUMENT_DEFINITION`
///
/// Reference: https://github.com/graphql/composite-schemas-spec/blob/main/spec/Section%202%20--%20Source%20Schema.md#is
pub struct IsDirective<'a> {
    pub field: &'a str,
}

impl<'a> ValueDeserialize<'a> for IsDirective<'a> {
    fn deserialize(input: cynic_parser_deser::DeserValue<'a>) -> Result<Self, cynic_parser_deser::Error> {
        let cynic_parser_deser::DeserValue::Object(obj) = input else {
            return Err(cynic_parser_deser::Error::unexpected_type(
                cynic_parser_deser::value::ValueType::Object,
                input,
            ));
        };

        let mut field_field = None;

        for field in obj.fields() {
            match field.name() {
                "field" => field_field = Some(field.value().deserialize()?),
                other => {
                    return Err(cynic_parser_deser::Error::UnknownField {
                        name: other.to_string(),
                        field_type: field.value().into(),
                        field_span: field.name_span(),
                    });
                }
            }
        }

        let Some(field) = field_field else {
            return Err(cynic_parser_deser::Error::MissingField {
                name: "field".to_owned(),
                object_span: obj.span(),
            });
        };

        Ok(IsDirective { field })
    }
}

#[cfg(test)]
mod tests {
    use super::{super::*, *};

    #[test]
    fn test_is_directive() {
        let doc = directive_test_document("@is(field: \"someField\")");
        let value = parse_from_test_document::<IsDirective<'_>>(&doc).unwrap();

        assert_eq!(value.field, "someField");
    }
}
