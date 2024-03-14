use super::{BoundFieldId, Location, OpInputValueId};

pub struct VariableDefinition {
    pub name: String,
    pub name_location: Location,
    pub default_value: Option<OpInputValueId>,
    /// Reserved input_value_id for the future value of this variable.
    /// Defaults to a Ref of the default value if it exists or Null otherwise.
    pub future_input_value_id: OpInputValueId,
    /// Keeping track of every field that used this variable.
    /// Used to know which variable is used by a given plan.
    /// Sorted.
    pub used_by: Vec<BoundFieldId>,
    pub r#type: schema::Type,
}
