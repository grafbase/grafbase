mod bind;
mod parse;
mod path;
mod selection_set;
mod walkers;

pub use bind::{BindError, BindResult};
pub use engine_parser::{types::OperationType, Pos};
pub use parse::{parse_operation, ParseError, UnboundOperation};
pub use path::{OperationPath, OperationPathSegment, ResolvedTypeCondition};
use schema::{FieldId, InputValueId, InterfaceId, ObjectId, Schema, UnionId};
pub use selection_set::{OperationSelection, OperationSelectionSet};
pub use walkers::*;

use crate::{
    execution::{StrId, Strings},
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OperationFieldId(u32);

#[derive(Debug)]
pub struct Operation {
    pub ty: OperationType,
    pub name: Option<String>,
    pub selection_set: OperationSelectionSet,
    pub fields: Vec<OperationField>,
    pub strings: Strings,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

#[derive(Debug)]
pub struct OperationField {
    pub name: StrId,
    pub position: usize,
    // probably needs a better name. it's the position for requested fields. For added fields,
    // it's the position of the query field that needed it.
    pub pos: Pos,
    // resolving fragments eagerly, it makes manipulating SelectionSet easier during planning.
    pub type_condition: Option<TypeCondition>,
    pub field_id: FieldId,
    pub arguments: Vec<OperationFieldArgument>,
}

#[derive(Debug)]
pub struct OperationFieldArgument {
    pub name_pos: Pos,
    pub input_value_id: InputValueId,
    pub value_pos: Pos,
    pub value: engine_value::Value,
}

impl Operation {
    /// Binds an unbound operation to a schema. All field names are mapped to their actual field
    /// id in the schema. At this stage the operation might not be resolvable but it should make
    /// sense given the schema types.
    pub fn bind(schema: &Schema, unbound_operation: UnboundOperation) -> BindResult<Self> {
        let ty = unbound_operation.definition.ty;
        let mut binder = bind::Binder::new(schema);
        let selection_set = binder.bind(unbound_operation.definition)?;
        Ok(Operation {
            ty,
            name: unbound_operation.name,
            selection_set,
            fields: binder.fields,
            strings: binder.strings,
        })
    }
}

impl std::ops::Index<OperationFieldId> for Operation {
    type Output = OperationField;

    fn index(&self, index: OperationFieldId) -> &Self::Output {
        &self.fields[index.0 as usize]
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
