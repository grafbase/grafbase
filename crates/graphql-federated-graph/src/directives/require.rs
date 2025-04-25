use cynic_parser_deser::{ConstDeserializer as _, ValueDeserialize};

/// The composite spec `@require` directive, but in the context of a federated schema. Not to be confused with the federation `@requires` directive.
///
/// Reference: https://github.com/graphql/composite-schemas-spec/blob/main/spec/Section%202%20--%20Source%20Schema.md#require
pub struct RequireDirective<'a> {
    pub field: &'a str,
}

impl<'a> ValueDeserialize<'a> for RequireDirective<'a> {
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

        Ok(RequireDirective { field })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directives::{directive_test_document, parse_from_test_document};

    #[test]
    fn test_require_directive() {
        let doc = directive_test_document("@require(field: \"someField\")");
        let value = parse_from_test_document::<RequireDirective<'_>>(&doc).unwrap();

        assert_eq!(value.field, "someField");
    }
}
