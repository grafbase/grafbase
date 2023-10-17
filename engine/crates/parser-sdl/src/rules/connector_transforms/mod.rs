use std::collections::HashSet;

use engine::registry::{
    type_kinds::{InputType, OutputType},
    InputObjectType, InterfaceType, MetaType, ObjectType, Registry,
};

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
                fields.remove(&field);
            }
            Some(MetaType::InputObject(InputObjectType { input_fields, .. })) => {
                input_fields.remove(&field);
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
        .iter()
        .filter_map(|(name, ty)| lookup.starting_type.is_match(name).then_some(ty))
        .collect::<HashSet<_>>();

    for segment in &lookup.path {
        let mut next_types = HashSet::new();
        for ty in current_types {
            if let Ok(output_type) = OutputType::try_from(ty) {
                next_types.extend(output_type.fields().filter_map(|field| {
                    segment
                        .is_match(&field.name)
                        .then(|| registry.lookup_expecting::<&MetaType>(&field.ty).ok())
                        .flatten()
                }));
            } else if let Ok(input_type) = InputType::try_from(ty) {
                next_types.extend(input_type.fields().filter_map(|field| {
                    segment
                        .is_match(&field.name)
                        .then(|| registry.lookup_expecting::<&MetaType>(&field.ty).ok())
                        .flatten()
                }));
            }
        }
        current_types = next_types;
    }

    let mut fields = HashSet::new();
    for ty in current_types {
        if let Ok(output_type) = OutputType::try_from(ty) {
            fields.extend(
                output_type
                    .fields()
                    .filter(|field| lookup.field.is_match(&field.name))
                    .map(|field| SelectedField {
                        ty: ty.name().to_string(),
                        field: field.name.clone(),
                    }),
            );
        } else if let Ok(input_type) = InputType::try_from(ty) {
            fields.extend(
                input_type
                    .fields()
                    .filter(|field| lookup.field.is_match(&field.name))
                    .map(|field| SelectedField {
                        ty: ty.name().to_string(),
                        field: field.name.clone(),
                    }),
            );
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use engine::registry::Registry;
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
