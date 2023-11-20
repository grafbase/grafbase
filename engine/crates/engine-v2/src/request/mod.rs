mod binder;
mod fields;
mod path;
mod selection_set;

use engine_parser::types::OperationDefinition;
pub use engine_parser::types::OperationType;
pub use fields::{OperationArgument, OperationFieldId, OperationFields, Pos, TypeCondition};
pub use path::{OperationPath, OperationPathSegment, ResolvedTypeCondition};
use schema::Schema;
pub use selection_set::{OperationSelection, OperationSelectionSet};

use crate::{
    execution::Strings,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

#[derive(Debug)]
pub struct Operation {
    pub ty: OperationType,
    pub selection_set: OperationSelectionSet,
    pub fields: OperationFields,
}

impl Operation {
    pub fn bind(schema: &Schema, operation_definition: OperationDefinition, strings: &mut Strings) -> Self {
        let mut fields = OperationFields::new();
        let ty = operation_definition.ty;
        let selection_set = binder::OperationBinder::new(schema, &mut fields, strings)
            .bind(operation_definition)
            .unwrap();
        Operation {
            ty,
            selection_set,
            fields,
        }
    }
}

impl ContextAwareDebug for Operation {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationDefinition")
            .field("ty", &self.ty)
            .field("selection_set", &ctx.debug(&self.selection_set))
            .finish()
    }
}
