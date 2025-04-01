use crate::wit::{Error, FieldDefinitionDirective, FieldOutput, FieldResolverGuest, Headers};

use super::{state, Component};

impl FieldResolverGuest for Component {
    fn resolve_field(
        headers: Headers,
        subgraph_name: String,
        directive: FieldDefinitionDirective,
        inputs: Vec<Vec<u8>>,
    ) -> Result<FieldOutput, Error> {
        let result =
            state::extension()?.resolve_field(headers.into(), &subgraph_name, (&directive).into(), (&inputs).into());

        result.map(Into::into).map_err(Into::into)
    }

    fn subscription_key(
        headers: Headers,
        subgraph_name: String,
        directive: FieldDefinitionDirective,
    ) -> Result<Option<Vec<u8>>, Error> {
        let result = state::extension()?.subscription_key(&headers.into(), &subgraph_name, (&directive).into())?;

        Ok(result)
    }

    fn resolve_subscription(
        headers: Headers,
        subgraph_name: String,
        directive: FieldDefinitionDirective,
    ) -> Result<(), Error> {
        let subscription =
            state::extension()?.resolve_subscription(headers.into(), &subgraph_name, (&directive).into())?;

        state::set_subscription(subscription);

        Ok(())
    }

    fn resolve_next_subscription_item() -> Result<Option<FieldOutput>, Error> {
        Ok(state::subscription()?.next()?.map(Into::into))
    }
}
