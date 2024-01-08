use crate::schema;
use async_graphql_parser::types::ServiceDocument;
use std::collections::HashSet;

impl From<ServiceDocument> for schema::Schema {
    fn from(value: ServiceDocument) -> Self {
        let mut fields: Vec<schema::SchemaField> = Vec::new();
        let mut query_type_name: Option<String> = None;
        let mut mutation_type_name = None;
        let mut subscription_type_name = None;
        let mut field_arguments = Vec::new();
        let mut input_objects = HashSet::new();

        for definition in value.definitions {
            match definition {
                async_graphql_parser::types::TypeSystemDefinition::Schema(schema_def) => {
                    if let Some(query) = schema_def.node.query {
                        query_type_name = Some(query.node.to_string());
                    }

                    if let Some(mutation) = schema_def.node.mutation {
                        mutation_type_name = Some(mutation.node.to_string());
                    }

                    if let Some(subscription) = schema_def.node.subscription {
                        subscription_type_name = Some(subscription.node.to_string());
                    }
                }
                async_graphql_parser::types::TypeSystemDefinition::Directive(_) => (),
                async_graphql_parser::types::TypeSystemDefinition::Type(typedef) => {
                    let type_name = typedef.node.name.node.as_str();

                    match typedef.node.kind {
                        async_graphql_parser::types::TypeKind::Enum(_)
                        | async_graphql_parser::types::TypeKind::Union(_)
                        | async_graphql_parser::types::TypeKind::Scalar => (),

                        async_graphql_parser::types::TypeKind::Object(obj) => {
                            for field in &obj.fields {
                                let field_name = field.node.name.node.as_str();
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field_name.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                    type_is_required: !field.node.ty.node.nullable,
                                });

                                for argument in &field.node.arguments {
                                    field_arguments.push(schema::FieldArgument {
                                        type_name: type_name.to_string(),
                                        field_name: field_name.to_string(),
                                        argument_name: argument.node.name.node.to_string(),
                                        base_type: extract_type_name(&argument.node.ty.node.base),
                                        is_required: !argument.node.ty.node.nullable,
                                        has_default: argument.node.default_value.is_some(),
                                    });
                                }
                            }
                        }
                        async_graphql_parser::types::TypeKind::Interface(iface) => {
                            for field in &iface.fields {
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                    type_is_required: !field.node.ty.node.nullable,
                                });
                            }
                        }
                        async_graphql_parser::types::TypeKind::InputObject(input_obj) => {
                            input_objects.insert(type_name.to_string());

                            for field in &input_obj.fields {
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type: extract_type_name(&field.node.ty.node.base),
                                    type_is_required: !field.node.ty.node.nullable,
                                });
                            }
                        }
                    }
                }
            }
        }

        fields.sort();
        field_arguments.sort();

        schema::Schema {
            fields,
            field_arguments,
            input_objects,
            query_type_name: query_type_name.unwrap_or_else(|| "Query".to_owned()),
            mutation_type_name: mutation_type_name.unwrap_or_else(|| "Mutation".to_owned()),
            subscription_type_name: subscription_type_name.unwrap_or_else(|| "Subscription".to_owned()),
        }
    }
}

fn extract_type_name(ty: &async_graphql_parser::types::BaseType) -> String {
    match ty {
        async_graphql_parser::types::BaseType::Named(name) => name.to_string(),
        async_graphql_parser::types::BaseType::List(inner) => extract_type_name(&inner.base),
    }
}
