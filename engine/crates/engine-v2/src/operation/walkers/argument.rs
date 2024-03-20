use schema::{InputValueDefinitionId, InputValueDefinitionWalker};

use crate::operation::{FieldArgumentId, QueryInputValueWalker};

use super::OperationWalker;

pub type FieldArgumentWalker<'a> = OperationWalker<'a, FieldArgumentId, InputValueDefinitionId>;

impl<'a> FieldArgumentWalker<'a> {
    pub fn value(&self) -> Option<QueryInputValueWalker<'a>> {
        let value = self.walk_with(&self.operation[self.as_ref().input_value_id], ());
        if value.is_undefined() {
            None
        } else {
            Some(value)
        }
    }
}

impl<'a> std::ops::Deref for FieldArgumentWalker<'a> {
    type Target = InputValueDefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl std::fmt::Debug for FieldArgumentWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
    }
}
