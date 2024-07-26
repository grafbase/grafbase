use serde::ser::SerializeMap;

use crate::execution::PlanWalker;

use super::query::QueryVariables;

pub(super) struct SubgraphVariables<'a, Input> {
    pub plan: PlanWalker<'a>,
    pub variables: &'a QueryVariables,
    pub inputs: Vec<(&'a str, Input)>,
}

impl<'a, Input> serde::Serialize for SubgraphVariables<'a, Input>
where
    Input: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.variables.len() + self.inputs.len()))?;
        for (name, input_value_id) in self.variables.iter() {
            let value = self.plan.walk_input_value(input_value_id);
            if !value.is_undefined() {
                map.serialize_entry(&name, &value)?;
            }
        }
        for (key, response_objects) in &self.inputs {
            map.serialize_entry(key, response_objects)?;
        }
        map.end()
    }
}
