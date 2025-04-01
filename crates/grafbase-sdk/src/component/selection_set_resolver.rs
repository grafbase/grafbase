use super::{Component, state};
use crate::wit::{
    Data, Error, Headers, SelectionSetResolverGuest,
    selection_set_resolver_types::{ArgumentsId, Field, FieldId},
};

impl SelectionSetResolverGuest for Component {
    fn prepare(root_field_id: FieldId, fields: Vec<Field>) -> Result<Vec<u8>, Error> {
        let result = state::extension()?.selection_set_resolver_prepare(crate::types::Field {
            fields: &fields,
            field: &fields[usize::from(root_field_id)],
        });

        result.map_err(Into::into)
    }

    fn resolve_query_or_mutation_field(
        headers: Headers,
        prepared: Vec<u8>,
        arguments: Vec<(ArgumentsId, Vec<u8>)>,
    ) -> Result<Data, Error> {
        let result = state::extension()?.selection_set_resolver_resolve(
            headers.into(),
            prepared,
            crate::types::ArgumentValues(&arguments),
        );

        result.map(Into::into).map_err(Into::into)
    }
}
