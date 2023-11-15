mod binder;
mod fields;
mod path;
mod selection_set;

pub use binder::OperationBinder;
use engine_parser::types::OperationDefinition;
pub use engine_parser::types::OperationType;
pub use fields::{
    OperationArgument, OperationField, OperationFieldId, OperationFields, OperationFieldsBuilder, Pos, TypeCondition,
};
pub use path::{OperationPath, OperationPathSegment, ResolvedTypeCondition};
use schema::Schema;
pub use selection_set::{OperationSelection, OperationSelectionSet};

use crate::{
    execution::ExecutionStrings,
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

#[derive(Debug)]
pub struct Operation {
    pub ty: OperationType,
    pub selection_set: OperationSelectionSet,
    pub fields: OperationFields,
    pub strings: ExecutionStrings,
}

impl Operation {
    pub fn build(schema: &Schema, operation_definition: OperationDefinition) -> Self {
        let mut strings = ExecutionStrings::new();
        let mut fields = OperationFields::builder(&mut strings);
        let ty = operation_definition.ty;
        let selection_set = OperationBinder {
            schema,
            fields: &mut fields,
        }
        .bind(operation_definition)
        .unwrap();
        let fields = fields.build();
        Operation {
            ty,
            selection_set,
            fields,
            strings,
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
