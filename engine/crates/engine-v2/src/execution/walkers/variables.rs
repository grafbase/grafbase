use engine_value::ConstValue;
use schema::{ListWrapping, SchemaWalker};

use crate::execution::{Variable, Variables};

#[derive(Clone, Copy)]
pub struct VariablesWalker<'a> {
    schema: SchemaWalker<'a, ()>,
    inner: &'a Variables<'a>,
}

impl<'a> VariablesWalker<'a> {
    pub fn new(schema: SchemaWalker<'a, ()>, inner: &'a Variables<'a>) -> Self {
        Self { schema, inner }
    }

    #[allow(clippy::panic)]
    pub fn unchecked_get(&self, name: &str) -> VariableWalker<'a> {
        VariableWalker {
            schema: self.schema,
            inner: self.inner.unchecked_get(name),
        }
    }
}

pub struct VariableWalker<'a> {
    schema: SchemaWalker<'a, ()>,
    inner: &'a Variable<'a>,
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
