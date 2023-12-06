use engine_value::ConstValue;
use schema::ListWrapping;

use crate::execution::{Variable, Variables};

use super::OperationWalker;

pub trait HasVariables {
    fn variables(&self) -> &Variables<'_>;
}

pub type VariablesWalker<'a> = OperationWalker<'a, &'a Variables<'a>>;
pub type VariableWalker<'a> = OperationWalker<'a, &'a Variable<'a>>;

impl<'a> VariablesWalker<'a> {
    pub fn get(&self, name: &str) -> Option<VariableWalker<'a>> {
        self.inner.get(name).map(|variable| self.walk(variable))
    }
}

impl<'a> VariableWalker<'a> {
    pub fn value(&self) -> &'a ConstValue {
        &self.inner.value
    }

    pub fn type_name(&self) -> String {
        let ty = &self.inner.definition.r#type;
        let mut name = self.schema.walk(ty.inner).name().to_string();
        if ty.wrapping.inner_is_required {
            name.push('!');
        }
        for list_wrapping in &ty.wrapping.list_wrapping {
            name = match list_wrapping {
                ListWrapping::RequiredList => format!("[{name}]!"),
                ListWrapping::NullableList => format!("[{name}]"),
            }
        }
        name
    }

    pub fn default_value(&self) -> &Option<ConstValue> {
        &self.inner.definition.default_value
    }
}
