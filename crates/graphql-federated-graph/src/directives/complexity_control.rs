use cynic_parser_deser::ValueDeserialize;

#[derive(ValueDeserialize)]
pub struct CostDirective {
    pub weight: i32,
}

impl CostDirective {
    pub fn definition() -> &'static str {
        indoc::indoc! {r#"
            directive @cost(weight: Int!) on
                ARGUMENT_DEFINITION
              | ENUM
              | FIELD_DEFINITION
              | INPUT_FIELD_DEFINITION
              | OBJECT
              | SCALAR
        "#}
    }
}

#[derive(ValueDeserialize, PartialEq, PartialOrd, Clone, Debug)]
#[deser(rename_all = "camelCase", default)]
pub struct ListSizeDirective {
    pub assumed_size: Option<u32>,
    pub slicing_arguments: Vec<String>,
    pub sized_fields: Vec<String>,
    #[deser(default = true)]
    pub require_one_slicing_argument: bool,
}

impl ListSizeDirective {
    pub fn definition() -> &'static str {
        indoc::indoc! {r#"
            directive @listSize(
              assumedSize: Int,
              slicingArguments: [String!],
              sizedFields: [String!],
              requireOneSlicingArgument: Boolean = true
            ) on FIELD_DEFINITION
        "#}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directives::parse_directive;

    #[test]
    fn test_parsing_cost() {
        let value = parse_directive::<CostDirective>("@cost(weight: 1)").unwrap();

        assert_eq!(value.weight, 1);
    }
}
