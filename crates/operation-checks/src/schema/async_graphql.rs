use super::WrappingTypes;
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
        let mut interface_implementations = Vec::new();

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
                            for implemented_interface in &obj.implements {
                                interface_implementations
                                    .push((type_name.to_string(), implemented_interface.node.to_string()));
                            }

                            for field in &obj.fields {
                                let field_name = field.node.name.node.as_str();
                                let (base_type, wrappers) = extract_type(&field.node.ty.node);
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field_name.to_string(),
                                    base_type,
                                    wrappers,
                                });

                                for argument in &field.node.arguments {
                                    let (base_type, wrappers) = extract_type(&argument.node.ty.node);
                                    field_arguments.push(schema::FieldArgument {
                                        type_name: type_name.to_string(),
                                        field_name: field_name.to_string(),
                                        argument_name: argument.node.name.node.to_string(),
                                        base_type,
                                        wrappers,
                                        has_default: argument.node.default_value.is_some(),
                                    });
                                }
                            }
                        }
                        async_graphql_parser::types::TypeKind::Interface(iface) => {
                            for implemented_interface in &iface.implements {
                                interface_implementations
                                    .push((type_name.to_string(), implemented_interface.node.to_string()));
                            }

                            for field in &iface.fields {
                                let (base_type, wrappers) = extract_type(&field.node.ty.node);
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type,
                                    wrappers,
                                });
                            }
                        }
                        async_graphql_parser::types::TypeKind::InputObject(input_obj) => {
                            input_objects.insert(type_name.to_string());

                            for field in &input_obj.fields {
                                let (base_type, wrappers) = extract_type(&field.node.ty.node);
                                fields.push(schema::SchemaField {
                                    type_name: type_name.to_string(),
                                    field_name: field.node.name.node.to_string(),
                                    base_type,
                                    wrappers,
                                });
                            }
                        }
                    }
                }
            }
        }

        fields.sort();
        field_arguments.sort();
        interface_implementations.sort();

        schema::Schema {
            fields,
            field_arguments,
            input_objects,
            interface_implementations,
            query_type_name: query_type_name.unwrap_or_else(|| "Query".to_owned()),
            mutation_type_name: mutation_type_name.unwrap_or_else(|| "Mutation".to_owned()),
            subscription_type_name: subscription_type_name.unwrap_or_else(|| "Subscription".to_owned()),
        }
    }
}

pub(crate) fn extract_type(top_level_ty: &async_graphql_parser::types::Type) -> (String, WrappingTypes) {
    let mut ty = top_level_ty;
    let mut wrapper_types = WrappingTypes::default();

    loop {
        match &ty.base {
            async_graphql_parser::types::BaseType::Named(name) => {
                wrapper_types.set_inner_nonnull(!ty.nullable);
                return (name.to_string(), wrapper_types);
            }
            async_graphql_parser::types::BaseType::List(inner) => {
                wrapper_types.push_list(!ty.nullable);
                ty = inner.as_ref();
            }
        }
    }
}
