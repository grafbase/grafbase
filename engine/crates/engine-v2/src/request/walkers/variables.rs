use engine_value::ConstValue;
use schema::ListWrapping;

use crate::request::VariableDefinitionId;

use super::OperationWalker;

pub type VariableDefinitionWalker<'a> = OperationWalker<'a, VariableDefinitionId>;

impl<'a> VariableDefinitionWalker<'a> {
    pub fn type_name(&self) -> String {
        let ty = &self.as_ref().r#type;
        let mut name = self.schema_walker.walk(ty.inner).name().to_string();
        if ty.wrapping.inner_is_required() {
            name.push('!');
        }
        for list_wrapping in ty.wrapping.list_wrappings() {
            name = match list_wrapping {
                ListWrapping::RequiredList => format!("[{name}]!"),
                ListWrapping::NullableList => format!("[{name}]"),
            }
        }
        name
    }

    pub fn default_value(&self) -> Option<&ConstValue> {
        self.as_ref().default_value.as_ref()
    }
}
