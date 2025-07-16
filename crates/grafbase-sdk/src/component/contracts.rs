use super::Component;
use crate::wit;

impl wit::ContractsGuest for Component {
    fn construct(
        _key: String,
        _directives: Vec<wit::Directive>,
        _subgraphs: Vec<wit::GraphqlSubgraph>,
    ) -> Result<wit::Contract, String> {
        todo!()
    }
}
