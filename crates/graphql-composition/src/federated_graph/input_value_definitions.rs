use super::*;

impl FederatedGraph {
    pub fn push_input_value_definition(
        &mut self,
        input_value_definition: InputValueDefinition,
    ) -> InputValueDefinitionId {
        let id = self.input_value_definitions.len().into();
        self.input_value_definitions.push(input_value_definition);
        id
    }
}
