use crate::federated_graph::{FieldId, InputValueDefinitionId};

#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub struct ListSize {
    pub assumed_size: Option<u32>,
    // Arguments on the current field to interpret as slice size
    pub slicing_arguments: Vec<InputValueDefinitionId>,
    // Fields on the child object that this size directive applies to
    pub sized_fields: Vec<FieldId>,
    pub require_one_slicing_argument: bool,
}
