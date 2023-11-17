use std::collections::HashSet;

use engine_parser::{
    types::{ConstDirective, TypeKind, TypeSystemDefinition},
    Positioned,
};

#[derive(Default)]
pub struct UsedDirectives {
    schema: HashSet<String>,
    r#type: HashSet<String>,
    object_field: HashSet<String>,
    interface_field: HashSet<String>,
    enum_value: HashSet<String>,
    input_value: HashSet<String>,
}

impl UsedDirectives {
    pub fn all(&self) -> HashSet<String> {
        let mut out = HashSet::new();
        out.extend(self.schema.clone());
        out.extend(self.r#type.clone());
        out.extend(self.object_field.clone());
        out.extend(self.interface_field.clone());
        out.extend(self.enum_value.clone());
        out.extend(self.input_value.clone());
        out
    }
}

pub fn parse_used_directives(schema: &str) -> engine::parser::Result<UsedDirectives> {
    let doc = super::parse_schema(schema)?;
    let mut used = UsedDirectives::default();
    fn directive_name(directive: Positioned<ConstDirective>) -> String {
        directive.node.name.node.to_string()
    }
    for definition in doc.definitions {
        match definition {
            TypeSystemDefinition::Schema(schema) => used
                .schema
                .extend(schema.node.directives.into_iter().map(directive_name)),
            TypeSystemDefinition::Type(ty) => {
                used.r#type.extend(ty.node.directives.into_iter().map(directive_name));
                match ty.node.kind {
                    TypeKind::Object(object) => {
                        for field in object.fields {
                            used.object_field
                                .extend(field.node.directives.into_iter().map(directive_name));
                            used.input_value.extend(
                                field
                                    .node
                                    .arguments
                                    .into_iter()
                                    .flat_map(|arg| arg.node.directives)
                                    .map(directive_name),
                            );
                        }
                    }
                    TypeKind::Interface(interface) => {
                        for field in interface.fields {
                            used.interface_field
                                .extend(field.node.directives.into_iter().map(directive_name));
                            used.input_value.extend(
                                field
                                    .node
                                    .arguments
                                    .into_iter()
                                    .flat_map(|arg| arg.node.directives)
                                    .map(directive_name),
                            );
                        }
                    }
                    TypeKind::Enum(r#enum) => {
                        for value in r#enum.values {
                            used.enum_value
                                .extend(value.node.directives.into_iter().map(directive_name));
                        }
                    }
                    TypeKind::InputObject(input_object) => {
                        for field in input_object.fields {
                            used.input_value
                                .extend(field.node.directives.into_iter().map(directive_name));
                        }
                    }
                    TypeKind::Union(_) | TypeKind::Scalar => (),
                }
            }
            TypeSystemDefinition::Directive(_) => (),
        }
    }
    Ok(used)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case::schema(
        r"
            extend schema @auth(rules: [ { allow: anonymous } ])
        ",
        &["specifiedBy", "auth"]
    )]
    #[case::model(
        r"
            type Post @model {
                name: String
            }
        ",
        &["specifiedBy", "model"]
    )]
    #[case::fields(
        r#"
            type Post @model @search {
              id: ID!
              blog: Blog @search
              content: String! @resolver(name: "text/summary")
              authors: [Author] @relation(name: "published")
            }

            type Author @model {
              id: ID!
              name: String!
              lastname: String! @default
              country: Country!
              posts: [Post] @relation(name: "published")
            }
        "#,
        &["specifiedBy", "model", "relation", "search", "default", "resolver"]
    )]
    fn test_parsed_used_directives(#[case] schema: &'static str, #[case] expected: &'static [&'static str]) {
        assert_eq!(
            parse_used_directives(schema).unwrap().all(),
            expected.iter().map(|s| (*s).to_string()).collect::<HashSet<_>>()
        );
    }
}
