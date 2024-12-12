use operation::InputValueContext;
use serde::ser::SerializeMap;
use walker::Walk;

use super::QueryVariables;

pub(crate) struct SubgraphGraphqlRequest<'a, Input> {
    pub query: &'a str,
    pub variables: SubgraphVariables<'a, Input>,
}

impl<Input> serde::Serialize for SubgraphGraphqlRequest<'_, Input>
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
    pub ctx: InputValueContext<'a>,
    pub variables: &'a QueryVariables,
    pub extra_variables: Vec<(&'a str, ExtraVariable)>,
}

impl<ExtraVariable> serde::Serialize for SubgraphVariables<'_, ExtraVariable>
where
    ExtraVariable: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        for (name, value_id) in self.variables.iter() {
            let value = value_id.walk(self.ctx);
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
