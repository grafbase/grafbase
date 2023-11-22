use std::collections::HashMap;

use engine_parser::Pos;
use engine_value::ConstValue;

use crate::{
    request::{Operation, VariableDefinition},
    response::GraphqlError,
};

#[derive(Debug, thiserror::Error)]
pub enum VariableError {
    #[error("Missing variable '{name}'")]
    MissingVariable { name: String, location: Pos },
}

impl From<VariableError> for GraphqlError {
    fn from(err: VariableError) -> Self {
        let locations = match err {
            VariableError::MissingVariable { location, .. } => vec![location],
        };
        GraphqlError {
            message: err.to_string(),
            locations,
            path: vec![],
        }
    }
}

pub struct Variables<'a> {
    inner: HashMap<String, Variable<'a>>,
}

pub struct Variable<'a> {
    pub value: ConstValue,
    pub definition: &'a VariableDefinition,
}

impl<'a> Variables<'a> {
    pub fn from_request(
        operation: &'a Operation,
        mut variables: engine_value::Variables,
    ) -> Result<Self, VariableError> {
        Ok(Self {
            inner: operation
                .variable_definitions
                .iter()
                .map(|definition| {
                    variables
                        .remove(&engine_value::Name::new(&definition.name))
                        .or_else(|| definition.default_value.clone())
                        .map(|value| (definition.name.clone(), Variable { value, definition }))
                        .ok_or_else(|| VariableError::MissingVariable {
                            name: definition.name.clone(),
                            location: definition.name_location,
                        })
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
        })
    }

    #[allow(clippy::panic)]
    pub fn unchecked_get(&self, name: &str) -> &Variable<'_> {
        self.inner
            .get(name)
            .unwrap_or_else(|| panic!("Missing variable '{name}'"))
    }
}
