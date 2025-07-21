use super::Component;
use crate::{component::state, wit};

impl wit::ContractsGuest for Component {
    fn construct(
        ctx: wit::SharedContext,
        key: String,
        directives: Vec<wit::Directive>,
        subgraphs: Vec<wit::GraphqlSubgraph>,
    ) -> Result<wit::Contract, String> {
        state::with_context(ctx, || {
            let directives = directives.iter().enumerate().map(Into::into).collect();
            let subgraphs = subgraphs.into_iter().map(Into::into).collect();

            state::extension()
                .map_err(|err| err.message)?
                .construct(key, directives, subgraphs)
                .map(Into::into)
                .map_err(|err| err.0.message)
        })
    }
}
