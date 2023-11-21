use std::collections::BTreeSet;

use engine::registry::{MetaType, Registry};
use engine_parser::types::Type;

#[derive(Debug, PartialEq)]
pub enum RequiredMigration {
    FieldMadeNonOptional { path: String },
}

impl std::fmt::Display for RequiredMigration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequiredMigration::FieldMadeNonOptional { path } => {
                write!(f, "field {path} cannot be made non-optional without a default value")
            }
        }
    }
}

pub fn required_migrations(from: &Registry, to: &Registry) -> Vec<RequiredMigration> {
    // FIXME: Simplify when https://github.com/rust-lang/rfcs/issues/1893 is accomplished.
    let from_type_names: BTreeSet<_> = from.types.keys().map(String::as_str).collect();
    let to_type_names: BTreeSet<_> = to.types.keys().map(String::as_str).collect();

    from_type_names
        .intersection(&to_type_names)
        .filter_map(|common_type_name| {
            let from_type = from
                .types
                .get(*common_type_name)
                .filter(|meta_type| meta_type.is_node());
            let to_type = to.types.get(*common_type_name).filter(|meta_type| meta_type.is_node());

            let from_create_input_type = from.types.get(&format!("{common_type_name}CreateInput"));
            let to_create_input_type = to.types.get(&format!("{common_type_name}CreateInput"));

            from_type
                .zip(from_create_input_type)
                .zip(to_type.zip(to_create_input_type))
        })
        .flat_map(
            |((from_type, _from_create_input_type), (to_type, to_create_input_type))| {
                let type_name = from_type.name();

                let MetaType::InputObject(to_create_input_object) = to_create_input_type else {
                    unreachable!("Impossible")
                };

                let to_create_input_fields = &to_create_input_object.input_fields;

                from_type
                    .fields()
                    .zip(to_type.fields())
                    .into_iter()
                    .flat_map(|(from_fields, to_fields)| {
                        let from_field_names: BTreeSet<_> = from_fields.keys().map(String::as_str).collect();
                        let to_field_names: BTreeSet<_> = to_fields.keys().map(String::as_str).collect();

                        from_field_names
                            .intersection(&to_field_names)
                            .filter_map(|&common_field_name| {
                                let to_has_default_value = to_create_input_fields
                                    .get(common_field_name)? // None if it's one of the built-in fields such as `id`, `createdAt` etc.
                                    .default_value
                                    .is_some();

                                let from_field = from_fields.get(common_field_name).unwrap();
                                let to_field = to_fields.get(common_field_name).unwrap();
                                let from_field_type = Type::new(&from_field.ty.to_string()).unwrap();
                                let to_field_type = Type::new(&to_field.ty.to_string()).unwrap();
                                if from_field_type.base == to_field_type.base
                                    && from_field_type.nullable
                                    && !to_field_type.nullable
                                    && !to_has_default_value
                                {
                                    Some(RequiredMigration::FieldMadeNonOptional {
                                        path: format!("{type_name}.{common_field_name}"),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            },
        )
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::RequiredMigration;

    #[rstest::rstest]
    #[case(
        r"
            type Product @model {
                id: ID!
                name: String!
            }
        ",
        r"
            type Product @model {
                id: ID!
                name: String
            }
        ",
        &[],
    )]
    #[case(
        r"
            type Product @model {
                id: ID!
                name: String
            }
        ",
        r"
            type Product @model {
                id: ID!
                name: String!
            }
        ",
        &[RequiredMigration::FieldMadeNonOptional { path: "Product.name".into() }]
    )]
    #[case(
        r"
            type Product @model {
                id: ID!
                name: String
            }
        ",
        r#"
            type Product @model {
                id: ID!
                name: String! @default(value: "default value")
            }
        "#,
        &[]
    )]
    fn test(#[case] from: &str, #[case] to: &str, #[case] expected_required_migrations: &[RequiredMigration]) {
        let from_registry = crate::parse_registry(from).unwrap();
        let to_registry = crate::parse_registry(to).unwrap();
        assert_eq!(
            super::required_migrations(&from_registry, &to_registry),
            expected_required_migrations
        );
    }
}
