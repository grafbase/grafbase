use crate::federated_graph::{InputValueDefinitionSet, SelectionSet, Value};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct AuthorizedDirective {
    pub fields: Option<SelectionSet>,
    pub node: Option<SelectionSet>,
    pub arguments: Option<InputValueDefinitionSet>,
    pub metadata: Option<Value>,
}
