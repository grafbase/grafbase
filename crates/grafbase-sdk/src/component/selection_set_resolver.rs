use super::{Component, state};
use crate::wit::{
    Data, Error, Headers, SelectionSetResolverGuest,
    selection_set_resolver_types::{ArgumentsId, Field, FieldId},
};

impl SelectionSetResolverGuest for Component {
    fn prepare(subgraph_name: String, root_field_id: FieldId, fields: Vec<Field>) -> Result<Vec<u8>, Error> {
        let result = state::extension()?.selection_set_resolver_prepare(
            &subgraph_name,
            crate::types::Field {
                fields: &fields,
                field: &fields[usize::from(root_field_id)],
            },
        );

        result.map_err(Into::into)
    }

    fn resolve_query_or_mutation_field(
        headers: Headers,
        subgraph_name: String,
        prepared: Vec<u8>,
        arguments: Vec<(ArgumentsId, Vec<u8>)>,
    ) -> Result<Data, Error> {
        let result = state::extension()?.selection_set_resolver_resolve(
            headers.into(),
            &subgraph_name,
            prepared,
            crate::types::ArgumentValues(&arguments),
        );

        result.map(Into::into).map_err(Into::into)
    }
}
