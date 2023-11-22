use engine_value::ConstValue;
use schema::ListWrapping;

use super::WalkerContext;
use crate::execution::{Variable, Variables};

pub struct VariablesWalker<'a> {
    pub(super) ctx: WalkerContext<'a, ()>,
    pub(super) inner: &'a Variables<'a>,
}

impl<'a> VariablesWalker<'a> {
    #[allow(clippy::panic)]
    pub fn unchecked_get(&self, name: &str) -> VariableWalker<'a> {
        VariableWalker {
            ctx: self.ctx,
            inner: self.inner.unchecked_get(name),
        }
    }
}

pub struct VariableWalker<'a> {
    ctx: WalkerContext<'a, ()>,
    inner: &'a Variable<'a>,
}

impl<'a> VariableWalker<'a> {
    pub fn value(&self) -> &'a ConstValue {
        &self.inner.value
    }

    pub fn type_name(&self) -> String {
        let ty = &self.inner.definition.r#type;
        let mut name = self.ctx.schema_walker.walk(ty.inner).name().to_string();
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
