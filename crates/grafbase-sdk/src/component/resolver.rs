use super::{Component, state};
use crate::wit;

impl wit::ResolverGuest for Component {
    fn prepare(
        ctx: wit::SharedContext,
        subgraph_name: String,
        directive: wit::Directive,
        root_field_id: wit::FieldId,
        fields: Vec<wit::Field>,
    ) -> Result<Vec<u8>, wit::Error> {
        state::with_context(ctx, || {
            let result = state::extension()?.prepare(crate::types::ResolvedField {
                subgraph_name: &subgraph_name,
                directive_name: &directive.name,
                directive_arguments: &directive.arguments,
                fields: fields.into(),
                root_field_ix: root_field_id as usize,
            });

            result.map_err(Into::into)
        })
    }

    fn resolve(
        ctx: wit::SharedContext,
        prepared: Vec<u8>,
        headers: wit::Headers,
        arguments: Vec<(wit::ArgumentsId, Vec<u8>)>,
    ) -> wit::Response {
        state::with_context(ctx, || {
            state::extension()
                .map(|ext| ext.resolve(&prepared, headers.into(), arguments.into()))
                .into()
        })
    }

    fn create_subscription(
        ctx: wit::SharedContext,
        prepared: Vec<u8>,
        headers: wit::Headers,
        arguments: Vec<(wit::ArgumentsId, Vec<u8>)>,
    ) -> Result<Option<Vec<u8>>, wit::Error> {
        state::with_context(ctx, || {
            // SAFETY: We keep prepared Vec alive with the subscription callback until it's called. We
            // also never modify the Vec at any point. Not providing a ref makes the API considerably
            // more tricky to work with.
            let slice: &'static [u8] = unsafe { std::mem::transmute(prepared.as_slice()) };
            let (key, callback) = state::extension()?.resolve_subscription(slice, headers.into(), arguments.into())?;

            state::set_subscription_callback(prepared, callback);

            Ok(key)
        })
    }

    fn resolve_next_subscription_item(ctx: wit::SharedContext) -> Result<Option<wit::SubscriptionItem>, wit::Error> {
        state::with_context(ctx, || {
            state::subscription().and_then(|sub| match sub.next() {
                Ok(Some(item)) => Ok(Some(item.into())),
                Ok(None) => Ok(None),
                Err(err) => Err(err.into()),
            })
        })
    }

    fn drop_subscription(ctx: wit::SharedContext) {
        state::with_context(ctx, || {
            state::drop_subscription();
        })
    }
}
