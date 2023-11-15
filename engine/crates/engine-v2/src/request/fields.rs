pub use engine_parser::Pos;
use schema::{FieldId, FieldTypeId, InterfaceId, ObjectId, StringId, UnionId};

use crate::execution::{ExecStringId, ExecutionStrings};

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
pub struct OperationField<Name = ExecStringId> {
    pub name: Name,
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
pub struct OperationFields {
    fields: Vec<OperationField>,
}

impl OperationFields {
    pub fn builder(strings: &mut ExecutionStrings) -> OperationFieldsBuilder<'_> {
        OperationFieldsBuilder {
            strings,
            fields: Vec::new(),
        }
    }

    pub fn into_builder(self, strings: &mut ExecutionStrings) -> OperationFieldsBuilder<'_> {
        OperationFieldsBuilder {
            strings,
            fields: self.fields,
        }
    }
}

impl std::ops::Index<OperationFieldId> for OperationFields {
    type Output = OperationField;

    fn index(&self, index: OperationFieldId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}

pub struct OperationFieldsBuilder<'a> {
    strings: &'a mut ExecutionStrings,
    fields: Vec<OperationField>,
}

impl<'a> OperationFieldsBuilder<'a> {
    pub fn get_or_intern(&mut self, value: &str) -> ExecStringId {
        self.strings.get_or_intern(value)
    }

    pub fn strings(&self) -> &ExecutionStrings {
        self.strings
    }

    pub fn push(&mut self, field: OperationField<&str>) -> OperationFieldId {
        let OperationField {
            name,
            position,
            pos,
            type_condition,
            field_id,
            arguments,
        } = field;
        self.fields.push(OperationField {
            name: self.strings.get_or_intern(name),
            position,
            pos,
            type_condition,
            field_id,
            arguments,
        });
        OperationFieldId((self.fields.len() - 1) as u32)
    }

    pub fn build(self) -> OperationFields {
        OperationFields { fields: self.fields }
    }
}

impl<'a> std::ops::Index<OperationFieldId> for OperationFieldsBuilder<'a> {
    type Output = OperationField;

    fn index(&self, index: OperationFieldId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}
