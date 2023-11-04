pub(super) mod create;
pub(super) mod filter;
pub(super) mod oneof;
pub(super) mod update;

use std::borrow::Cow;

use engine::registry::MetaInputValue;
use postgres_connector_types::database_definition::TableColumnWalker;

fn input_value_from_column(column: TableColumnWalker<'_>, oneof: bool) -> MetaInputValue {
    let mut client_type = column
        .graphql_type()
        .expect("unsupported types are filtered out at this point");

    // Oneof types can't enforce arguments, the runtime expects one of the arguments to be
    // defined. For nested input types, we must enforce any argument that cannot be null.
    if !oneof && !column.nullable() {
        client_type = Cow::from(format!("{client_type}!"));
    }

    MetaInputValue::new(column.client_name(), client_type.as_ref())
        .with_rename(Some(column.database_name().to_string()))
}
