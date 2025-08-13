use super::{Component, state};
use crate::wit;

impl wit::ResolverGuest for Component {
    fn prepare(
        event_queue: wit::EventQueue,
        subgraph_name: String,
        directive: wit::Directive,
        root_field_id: wit::FieldId,
        fields: Vec<wit::Field>,
    ) -> Result<Vec<u8>, wit::Error> {
        state::with_event_queue(event_queue, || {
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
        event_queue: wit::EventQueue,
        ctx: wit::AuthorizedOperationContext,
        prepared: Vec<u8>,
        headers: wit::Headers,
        arguments: Vec<(wit::ArgumentsId, Vec<u8>)>,
    ) -> wit::Response {
        state::with_event_queue(event_queue, || {
            state::extension()
                .map(|ext| ext.resolve(&(ctx.into()), &prepared, headers.into(), arguments.into()))
                .into()
        })
    }

    fn create_subscription(
        event_queue: wit::EventQueue,
        ctx: wit::AuthorizedOperationContext,
        prepared: Vec<u8>,
        headers: wit::Headers,
        arguments: Vec<(wit::ArgumentsId, Vec<u8>)>,
    ) -> Result<Option<Vec<u8>>, wit::Error> {
        state::set_event_queue(event_queue);
        // SAFETY: We keep prepared Vec alive with the subscription callback until it's called. We
        // also never modify the Vec at any point. Not providing a ref makes the API considerably
        // more tricky to work with.
        let prepared_ref: &'static [u8] = unsafe { std::mem::transmute(prepared.as_slice()) };
        let ctx: crate::types::AuthorizedOperationContext = ctx.into();
        let ctx = Box::new(ctx);
        let ctx_ref: &'static crate::types::AuthorizedOperationContext = unsafe { std::mem::transmute(ctx.as_ref()) };
        let (key, callback) =
            state::extension()?.resolve_subscription(ctx_ref, prepared_ref, headers.into(), arguments.into())?;

        state::set_subscription_callback(ctx, prepared, callback);

        Ok(key)
    }

    fn resolve_next_subscription_item() -> Result<Option<wit::SubscriptionItem>, wit::Error> {
        state::subscription().and_then(|sub| match sub.next() {
            Ok(Some(item)) => Ok(Some(item.into())),
            Ok(None) => Ok(None),
            Err(err) => Err(err.into()),
        })
    }

    fn drop_subscription() {
        state::drop_subscription();
        state::drop_event_queue();
    }
}
