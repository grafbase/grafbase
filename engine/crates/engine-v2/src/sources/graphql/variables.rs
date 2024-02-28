use serde::ser::SerializeMap;

use crate::{plan::PlanVariable, response::ResponseBoundaryObjectsView};

pub(super) struct OutboundVariables<'a> {
    pub variables: Vec<PlanVariable<'a>>,
    pub inputs: Vec<(&'a str, ResponseBoundaryObjectsView<'a>)>,
}

impl<'a> OutboundVariables<'a> {
    pub fn new(variables: Vec<PlanVariable<'a>>) -> Self {
        Self {
            variables,
            inputs: Vec::new(),
        }
    }
}

impl<'a> serde::Serialize for OutboundVariables<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.variables.len() + self.inputs.len()))?;
        for variable in &self.variables {
            let value = variable.value();
            if !value.is_undefined() {
                map.serialize_entry(variable.name(), &value)?;
            }
        }
        for (key, response_objects) in &self.inputs {
            map.serialize_entry(key, response_objects)?;
        }
        map.end()
    }
}
