use cynic_parser_deser::ValueDeserialize;

#[derive(ValueDeserialize)]
pub struct DeprecatedDirective<'a> {
    pub reason: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directives::{directive_test_document, parse_from_test_document};

    #[test]
    fn test_parsing_no_reason() {
        let doc = directive_test_document("@deprecated");
        let value = parse_from_test_document::<DeprecatedDirective<'_>>(&doc).unwrap();

        assert_eq!(value.reason, None);
    }

    #[test]
    fn test_parsing_with_reason() {
        let doc = directive_test_document("@deprecated(reason: \"because I wanted to\")");
        let value = parse_from_test_document::<DeprecatedDirective<'_>>(&doc).unwrap();

        assert_eq!(value.reason, None);
    }
}
