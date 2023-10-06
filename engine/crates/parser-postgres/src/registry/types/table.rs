use crate::registry::context::{InputContext, OutputContext};
use engine::registry::{
    resolvers::{transformer::Transformer, Resolver},
    Constraint, InputObjectType, MetaField, MetaInputValue, ObjectType,
};
use postgres_types::database_definition::{DatabaseType, ScalarType, TableWalker};
use std::borrow::Cow;

pub(super) fn generate(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    direction_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.type_name(table.client_name());
    let edge_type_name = register_edge_type(input_ctx, table, &type_name, output_ctx);

    register_orderby_input(input_ctx, direction_type, table, output_ctx);
    register_connection_type(input_ctx, table, &edge_type_name, output_ctx);

    output_ctx.with_object_type(&type_name, table.id(), |builder| {
        for column in table.columns() {
            let client_type = column
                .graphql_type()
                .expect("forgot to filter unsupported types before generating");

            let client_type = if column.nullable() {
                client_type.to_string()
            } else {
                format!("{client_type}!")
            };

            let mut field = MetaField::new(column.client_name(), client_type.as_ref());
            field.mapped_name = Some(column.database_name().to_string());

            field.resolver = Resolver::Transformer(Transformer::Select {
                key: column.database_name().to_string(),
            });

            let extra_transformer = match column.database_type() {
                DatabaseType::Scalar(ScalarType::Bytea) => Some(Transformer::BytesToBase64),
                DatabaseType::Scalar(ScalarType::ByteaArray) => Some(Transformer::ByteArrayToBase64Array),
                DatabaseType::Enum(_) => Some(Transformer::RemoteEnum),
                _ => None,
            };

            if let Some(transformer) = extra_transformer {
                field.resolver = field.resolver.and_then(transformer);
            }

            builder.push_scalar_field(field, column.id());
        }

        for relation in table.relations() {
            #[allow(clippy::if_not_else)]
            let field = if !relation.is_referenced_row_unique() {
                let connection_type_name = input_ctx.connection_type_name(relation.referenced_table().client_name());

                let mut field = MetaField::new(relation.client_field_name(), connection_type_name);

                field
                    .args
                    .insert(String::from("first"), MetaInputValue::new("first", "Int"));

                field
                    .args
                    .insert(String::from("last"), MetaInputValue::new("last", "Int"));

                field
                    .args
                    .insert(String::from("before"), MetaInputValue::new("before", "String"));

                field
                    .args
                    .insert(String::from("after"), MetaInputValue::new("after", "String"));

                let order_by_type = input_ctx.orderby_input_type_name(relation.referenced_table().client_name());

                field.args.insert(
                    String::from("orderBy"),
                    MetaInputValue::new("orderBy", format!("[{order_by_type}!]")),
                );

                field.resolver = Resolver::Transformer(Transformer::Select {
                    key: relation.client_field_name(),
                })
                .and_then(Transformer::PostgresSelectionData {
                    directive_name: input_ctx.directive_name().to_string(),
                    table_id: relation.referenced_table().id(),
                })
                .and_then(Transformer::PostgresPageInfo);

                field
            } else {
                let client_type = relation.client_type();
                let client_type = input_ctx.type_name(&client_type);

                let client_type = if relation.nullable() {
                    client_type
                } else {
                    Cow::Owned(format!("{client_type}!"))
                };

                let mut field = MetaField::new(relation.client_field_name(), client_type.as_ref());

                field.resolver = Resolver::Transformer(Transformer::Select {
                    key: relation.client_field_name(),
                });

                field
            };

            builder.push_relation_field(field, relation.id());
        }

        for constraint in table.unique_constraints() {
            let fields = constraint
                .columns()
                .map(|column| column.table_column().client_name().to_string())
                .collect();

            builder.push_constraint(Constraint::unique(constraint.name().to_string(), fields));
        }
    });
}

fn register_connection_type(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    edge_type_name: &str,
    output_ctx: &mut OutputContext,
) {
    let connection_type_name = input_ctx.connection_type_name(table.client_name());

    output_ctx.with_object_type(&connection_type_name, table.id(), |builder| {
        let field = MetaField::new("edges", format!("[{edge_type_name}]!"));

        builder.push_non_mapped_scalar_field(field);

        let type_name = input_ctx.type_name("PageInfo");
        let page_info = MetaField::new("pageInfo", format!("{type_name}!"));

        builder.push_non_mapped_scalar_field(page_info);
    });
}

fn register_edge_type(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    type_name: &str,
    output_ctx: &mut OutputContext,
) -> String {
    let edge_type_name = input_ctx.edge_type_name(table.client_name());
    let node = MetaField::new("node", format!("{type_name}!"));

    let mut cursor = MetaField::new("cursor", String::from("String!"));
    cursor.resolver = Resolver::Transformer(Transformer::PostgresCursor);

    output_ctx.create_object_type(ObjectType::new(edge_type_name.clone(), [node, cursor]));

    edge_type_name.clone()
}

fn register_orderby_input(
    input_ctx: &InputContext<'_>,
    direction_type: &str,
    table: TableWalker<'_>,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.orderby_input_type_name(table.client_name());

    let input_fields = table
        .columns()
        .map(|column| MetaInputValue::new(column.client_name().to_string(), direction_type));

    let input_object = InputObjectType::new(type_name.to_string(), input_fields).with_oneof(true);

    output_ctx.create_input_type(input_object);
}
