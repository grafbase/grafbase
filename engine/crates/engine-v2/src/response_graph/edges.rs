pub use engine_parser::Pos;
use schema::{FieldId, FieldTypeId, InterfaceId, ObjectId, StringId, UnionId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FieldEdgeId(pub(super) u32);

// Maybe an enum? Need to support `__typename`
pub struct FieldEdge {
    pub name: FieldName,
    // probably needs a better name. it's the position for requested fields. For added fields,
    // it's the position of the query field that needed it.
    pub pos: Pos,
    // resolving fragments eagerly, it makes manipulating SelectionSet easier during planning.
    pub type_condition: Option<TypeCondition>,
    pub field_id: FieldId,
    pub arguments: Vec<Argument>,
}

pub struct Argument {
    pub name_pos: Pos,
    pub name: StringId,
    pub type_id: FieldTypeId,
    pub value_pos: Pos,
    pub value: engine_value::Value,
}

pub struct ResponseGraphEdges {
    fields: Vec<FieldEdge>,
    field_names: lasso::Rodeo<FieldName>,
}

impl ResponseGraphEdges {
    pub fn builder() -> ResponseGraphEdgesBuilder {
        ResponseGraphEdgesBuilder {
            field_names: lasso::Rodeo::new(),
            fields: Vec::new(),
        }
    }

    pub(super) fn intern_field_name(&mut self, value: &str) -> FieldName {
        self.field_names.get_or_intern(value)
    }
}

impl std::ops::Index<FieldEdgeId> for ResponseGraphEdges {
    type Output = FieldEdge;

    fn index(&self, index: FieldEdgeId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}

impl std::ops::Index<FieldName> for ResponseGraphEdges {
    type Output = str;

    fn index(&self, index: FieldName) -> &Self::Output {
        &self.field_names[index]
    }
}

pub struct ResponseGraphEdgesBuilder {
    field_names: lasso::Rodeo<FieldName>,
    fields: Vec<FieldEdge>,
}

impl ResponseGraphEdgesBuilder {
    pub fn intern_field_name(&mut self, value: &str) -> FieldName {
        self.field_names.get_or_intern(value)
    }

    pub fn push_field(
        &mut self,
        name: &str,
        pos: Pos,
        field_id: FieldId,
        type_condition: Option<TypeCondition>,
        arguments: Vec<Argument>,
    ) -> FieldEdgeId {
        let name = self.field_names.get_or_intern(name);
        self.fields.push(FieldEdge {
            name,
            pos,
            type_condition,
            field_id,
            arguments,
        });
        FieldEdgeId((self.fields.len() - 1) as u32)
    }

    pub fn push_internal_field(
        &mut self,
        name: &str,
        pos: Pos,
        field_id: FieldId,
        type_condition: Option<TypeCondition>,
        arguments: Vec<Argument>,
    ) -> (FieldEdgeId, FieldName) {
        let name = self.field_names.get_or_intern(name);
        self.fields.push(FieldEdge {
            name,
            pos,
            type_condition,
            field_id,
            arguments,
        });
        (FieldEdgeId((self.fields.len() - 1) as u32), name)
    }

    pub fn build(self) -> ResponseGraphEdges {
        let ResponseGraphEdgesBuilder {
            field_names, fields, ..
        } = self;
        ResponseGraphEdges { fields, field_names }
    }
}

impl std::ops::Index<FieldEdgeId> for ResponseGraphEdgesBuilder {
    type Output = FieldEdge;

    fn index(&self, index: FieldEdgeId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldName(u32);

// Reserving the 4 upper bits for flags which still leaves 268 millions ids.
const ID_MASK: usize = 0x0F_FF_FF_FF;

unsafe impl lasso::Key for FieldName {
    fn into_usize(self) -> usize {
        self.0 as usize
    }

    fn try_from_usize(int: usize) -> Option<Self> {
        if int < ID_MASK {
            Some(Self(int as u32))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use lasso::Key;

    use super::*;

    #[test]
    fn field_name_value_in_range() {
        let key = FieldName::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = FieldName::try_from_usize(ID_MASK - 1).unwrap();
        assert_eq!(key.into_usize(), ID_MASK - 1);
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = FieldName::try_from_usize(ID_MASK);
        assert!(key.is_none());

        let key = FieldName::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
