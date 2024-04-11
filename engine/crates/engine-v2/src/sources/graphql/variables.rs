use serde::ser::SerializeMap;

use crate::{plan::PlanWalker, response::ResponseBoundaryObjectsView};

use super::query::QueryVariables;

pub(super) struct SubgraphVariables<'a> {
    pub plan: PlanWalker<'a>,
    pub variables: &'a QueryVariables,
    pub inputs: Vec<(&'a str, ResponseBoundaryObjectsView<'a>)>,
}

impl<'a> serde::Serialize for SubgraphVariables<'a> {
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
