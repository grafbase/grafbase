pub use engine_parser::Pos;
use schema::{FieldId, FieldTypeId, InterfaceId, ObjectId, StringId, UnionId};

use crate::execution::StrId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct OperationFieldId(pub(super) u32);

// Maybe an enum? Need to support `__typename`
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
    pub arguments: Vec<OperationArgument>,
}

#[derive(Debug)]
pub struct OperationArgument {
    pub name_pos: Pos,
    pub name: StringId,
    pub type_id: FieldTypeId,
    pub value_pos: Pos,
    pub value: engine_value::Value,
}

#[derive(Debug)]
pub struct OperationFields(Vec<OperationField>);

impl OperationFields {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, field: OperationField) -> OperationFieldId {
        self.0.push(field);
        OperationFieldId((self.0.len() - 1) as u32)
    }
}

impl std::ops::Index<OperationFieldId> for OperationFields {
    type Output = OperationField;

    fn index(&self, index: OperationFieldId) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}
