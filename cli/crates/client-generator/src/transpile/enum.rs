use async_graphql_parser::types::EnumType;

use crate::r#enum::{Enum, EnumVariant};
use crate::statement::Export;

pub(crate) fn generate<'a>(name: &'a str, description: Option<&'a str>, obj: &'a EnumType) -> Export<'a> {
    let mut r#enum = Enum::new(name);

    for value in &obj.values {
        let mut variant = EnumVariant::new(value.node.value.node.as_str());

        if let Some(ref comment) = value.node.description {
            variant.description(comment.node.as_str());
        }

        r#enum.push_variant(variant);
    }

    let mut result = Export::new(r#enum);

    if let Some(description) = description {
        result.description(description);
    }

    result
}
