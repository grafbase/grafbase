use serde::ser::SerializeMap;

use crate::execution::PlanWalker;

use super::QueryVariables;

pub(crate) struct SubgraphGraphqlRequest<'a, Input> {
    pub query: &'a str,
    pub variables: SubgraphVariables<'a, Input>,
}

impl<'a, Input> serde::Serialize for SubgraphGraphqlRequest<'a, Input>
where
    Input: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("query", self.query)?;
        map.serialize_entry("variables", &self.variables)?;
        map.end()
    }
}

pub(crate) struct SubgraphVariables<'a, ExtraVariable> {
    pub plan: PlanWalker<'a>,
    pub variables: &'a QueryVariables,
    pub extra_variables: Vec<(&'a str, ExtraVariable)>,
}

impl<'a, Input> serde::Serialize for SubgraphVariables<'a, Input>
where
    Input: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.variables.len() + self.extra_variables.len()))?;
        for (name, input_value_id) in self.variables.iter() {
            let value = self.plan.walk_input_value(input_value_id);
            if !value.is_undefined() {
                map.serialize_entry(&name, &value)?;
            }
        }
        for (key, response_objects) in &self.extra_variables {
            map.serialize_entry(key, response_objects)?;
        }
        map.end()
    }
}
