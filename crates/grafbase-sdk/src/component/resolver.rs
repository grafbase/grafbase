use super::{Component, state};
use crate::wit::{ArgumentsId, Directive, Error, Field, FieldId, Headers, ResolverGuest, Response, SubscriptionItem};

impl ResolverGuest for Component {
    fn prepare(
        subgraph_name: String,
        directive: Directive,
        root_field_id: FieldId,
        fields: Vec<Field>,
    ) -> Result<Vec<u8>, Error> {
        let result = state::extension()?.prepare(crate::types::ResolvedField {
            subgraph_name: subgraph_name.into(),
            directive_name: directive.name.into(),
            directive_arguments: directive.arguments.into(),
            fields: fields.into(),
            root_field_ix: root_field_id as usize,
        });

        result.map_err(Into::into)
    }

    fn resolve(prepared: Vec<u8>, headers: Headers, arguments: Vec<(ArgumentsId, Vec<u8>)>) -> Response {
        state::extension()
            .map(|ext| ext.resolve(&prepared, headers.into(), arguments.into()))
            .into()
    }

    fn create_subscription(
        prepared: Vec<u8>,
        headers: Headers,
        arguments: Vec<(ArgumentsId, Vec<u8>)>,
    ) -> Result<Option<Vec<u8>>, Error> {
        // SAFETY: We keep prepared Vec alive with the subscription callback until it's called. We
        // also never modify the Vec at any point. Not providing a ref makes the API considerably
        // more tricky to work with.
        let slice: &'static [u8] = unsafe { std::mem::transmute(prepared.as_slice()) };
        let (key, callback) = state::extension()?.resolve_subscription(slice, headers.into(), arguments.into())?;
        state::set_subscription_callback(prepared, callback);
        Ok(key)
    }

    fn resolve_next_subscription_item() -> Result<Option<SubscriptionItem>, Error> {
        state::subscription().and_then(|sub| match sub.next() {
            Ok(Some(item)) => Ok(Some(item.into())),
            Ok(None) => Ok(None),
            Err(err) => Err(err.into()),
        })
    }

    fn drop_subscription() {
        state::drop_subscription();
    }
}
