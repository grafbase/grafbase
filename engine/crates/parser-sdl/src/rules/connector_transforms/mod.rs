use std::collections::HashSet;

use engine::registry::{InputObjectType, InterfaceType, MetaType, ObjectType, Registry};

use self::field_lookup::FieldLookup;

mod field_lookup;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Transforms {
    exclude: Vec<FieldLookup>,
    pub prefix_types: Option<String>,
}

pub fn run_transforms(registry: &mut Registry, transforms: &Transforms) {
    let fields_to_remove = transforms
        .exclude
        .iter()
        .flat_map(|exclude| lookup_fields(registry, exclude))
        .collect::<HashSet<_>>();

    for SelectedField { ty, field } in fields_to_remove {
        match registry.types.get_mut(&ty) {
            Some(MetaType::Object(ObjectType { fields, .. }) | MetaType::Interface(InterfaceType { fields, .. })) => {
                fields.swap_remove(&field);
            }
            Some(MetaType::InputObject(InputObjectType { input_fields, .. })) => {
                input_fields.swap_remove(&field);
            }
            _ => {}
        }
    }

    registry.remove_unused_types();
}

#[derive(PartialEq, Eq, Hash)]
struct SelectedField {
    ty: String,
    field: String,
}

fn lookup_fields(registry: &Registry, lookup: &FieldLookup) -> HashSet<SelectedField> {
    let mut current_types = registry
        .types
        .keys()
        .filter(|name| lookup.starting_type.is_match(name))
        .collect::<HashSet<_>>();

    for segment in &lookup.path {
        let mut next_types = HashSet::new();
        for type_name in current_types {
            let ty = registry.types.get(type_name).unwrap();
            match ty {
                MetaType::Object(_) | MetaType::Interface(_) => {
                    next_types.extend(ty.fields().unwrap().keys().filter(|name| segment.is_match(name)));
                }
                MetaType::InputObject(input) => {
                    next_types.extend(input.input_fields.keys().filter(|name| segment.is_match(name)));
                }
                _ => {}
            }
        }
        current_types = next_types;
    }

    let mut fields = HashSet::new();
    for type_name in current_types {
        let ty = registry.types.get(type_name).unwrap();
        match ty {
            MetaType::Object(_) | MetaType::Interface(_) => fields.extend(
                ty.fields()
                    .unwrap()
                    .values()
                    .filter(|field| lookup.field.is_match(&field.name))
                    .map(|field| SelectedField {
                        ty: ty.name().to_string(),
                        field: field.name.clone(),
                    }),
            ),
            MetaType::InputObject(input) => {
                fields.extend(
                    input
                        .input_fields
                        .values()
                        .filter(|field| lookup.field.is_match(&field.name))
                        .map(|field| SelectedField {
                            ty: ty.name().to_string(),
                            field: field.name.clone(),
                        }),
                );
            }
            _ => {}
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use engine::registry::Registry;
    use engine::registry::RegistrySdlExt;
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    fn do_exclude_test(input: &str) -> String {
        let mut registry = registry();
        run_transforms(
            &mut registry,
            &Transforms::deserialize(json!({"exclude": [input]})).unwrap(),
        );
        registry.export_sdl(false)
    }

    #[test]
    fn test_excluding_specific_field() {
        insta::assert_snapshot!(do_exclude_test("User.email"), @r###"
        type Account {
        	id: ID!
        	email: String!
        }
        type Other {
        	id: ID!
        }
        type Query {
        	user: User
        	users: [User]
        	other: Other
        }
        type User {
        	id: ID!
        	name: String!
        	account: Account!
        }
        schema {
        	query: Query
        }
        "###);
    }

    #[test]
    fn test_excluding_removes_unused_types() {
        insta::assert_snapshot!(do_exclude_test("User.account"), @r###"
        type Other {
        	id: ID!
        }
        type Query {
        	user: User
        	users: [User]
        	other: Other
        }
        type User {
        	id: ID!
        	name: String!
        }
        schema {
        	query: Query
        }
        "###);
    }

    #[test]
    fn test_excluding_wildcards() {
        insta::assert_snapshot!(do_exclude_test("*.id"), @r###"
        type Account {
        	email: String!
        }
        type Other {
        }
        type Query {
        	user: User
        	users: [User]
        	other: Other
        }
        type User {
        	account: Account!
        	name: String!
        }
        schema {
        	query: Query
        }
        "###);
    }

    #[test]
    fn test_excluding_nested_wildcards() {
        insta::assert_snapshot!(do_exclude_test("Query.*.id"), @r###"
        type Account {
        	id: ID!
        	email: String!
        }
        type Other {
        }
        type Query {
        	user: User
        	users: [User]
        	other: Other
        }
        type User {
        	account: Account!
        	name: String!
        }
        schema {
        	query: Query
        }
        "###);
    }

    #[test]
    fn test_excluding_choices() {
        insta::assert_snapshot!(do_exclude_test("Query.{user,users}.name"), @r###"
        type Account {
        	id: ID!
        	email: String!
        }
        type Other {
        	id: ID!
        }
        type Query {
        	user: User
        	users: [User]
        	other: Other
        }
        type User {
        	id: ID!
        	account: Account!
        }
        schema {
        	query: Query
        }
        "###);
    }

    fn registry() -> Registry {
        crate::to_parse_result_with_variables(
            r#"
          type User {
            id: ID!
            name: String!
            account: Account!
          }

          type Account {
            id: ID!
            email: String!
          }

          type Other {
            id: ID!
          }

          extend type Query {
            user: User @resolver(name: "whatever")
            users: [User]  @resolver(name: "whatever")
            other: Other @resolver(name: "whatever")
          }
        "#,
            &HashMap::default(),
        )
        .unwrap()
        .registry
    }
}
