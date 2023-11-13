pub use engine_parser::Pos;
use schema::{FieldId, FieldTypeId, InterfaceId, ObjectId, StringId, UnionId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ResponseFieldId(pub(super) u32);

// Maybe an enum? Need to support `__typename`
pub struct ResponseField {
    pub name: ResponseStringId,
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

pub struct ResponseFields {
    fields: Vec<ResponseField>,
    strings: lasso::Rodeo<ResponseStringId>,
}

impl ResponseFields {
    pub fn builder() -> ResponseFieldsBuilder {
        ResponseFieldsBuilder {
            strings: lasso::Rodeo::new(),
            fields: Vec::new(),
        }
    }

    pub(super) fn intern_field_name(&mut self, value: &str) -> ResponseStringId {
        self.strings.get_or_intern(value)
    }
}

impl std::ops::Index<ResponseFieldId> for ResponseFields {
    type Output = ResponseField;

    fn index(&self, index: ResponseFieldId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}

impl std::ops::Index<ResponseStringId> for ResponseFields {
    type Output = str;

    fn index(&self, index: ResponseStringId) -> &Self::Output {
        &self.strings[index]
    }
}

pub struct ResponseFieldsBuilder {
    strings: lasso::Rodeo<ResponseStringId>,
    fields: Vec<ResponseField>,
}

impl ResponseFieldsBuilder {
    pub fn intern_field_name(&mut self, value: &str) -> ResponseStringId {
        self.strings.get_or_intern(value)
    }

    pub fn push_field(
        &mut self,
        name: &str,
        pos: Pos,
        field_id: FieldId,
        type_condition: Option<TypeCondition>,
        arguments: Vec<Argument>,
    ) -> ResponseFieldId {
        let name = self.strings.get_or_intern(name);
        self.fields.push(ResponseField {
            name,
            pos,
            type_condition,
            field_id,
            arguments,
        });
        ResponseFieldId((self.fields.len() - 1) as u32)
    }

    pub fn push_internal_field(
        &mut self,
        name: &str,
        pos: Pos,
        field_id: FieldId,
        type_condition: Option<TypeCondition>,
        arguments: Vec<Argument>,
    ) -> (ResponseFieldId, ResponseStringId) {
        let name = self.strings.get_or_intern(name);
        self.fields.push(ResponseField {
            name,
            pos,
            type_condition,
            field_id,
            arguments,
        });
        (ResponseFieldId((self.fields.len() - 1) as u32), name)
    }

    pub fn build(self) -> ResponseFields {
        let ResponseFieldsBuilder { strings, fields, .. } = self;
        ResponseFields { fields, strings }
    }
}

impl std::ops::Index<ResponseFieldId> for ResponseFieldsBuilder {
    type Output = ResponseField;

    fn index(&self, index: ResponseFieldId) -> &Self::Output {
        &self.fields[index.0 as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResponseStringId(u32);

// Reserving the 4 upper bits for flags which still leaves 268 millions ids.
const ID_MASK: usize = 0x0F_FF_FF_FF;

unsafe impl lasso::Key for ResponseStringId {
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
        let key = ResponseStringId::try_from_usize(0).unwrap();
        assert_eq!(key.into_usize(), 0);

        let key = ResponseStringId::try_from_usize(ID_MASK - 1).unwrap();
        assert_eq!(key.into_usize(), ID_MASK - 1);
    }

    #[test]
    fn field_name_value_out_of_range() {
        let key = ResponseStringId::try_from_usize(ID_MASK);
        assert!(key.is_none());

        let key = ResponseStringId::try_from_usize(u32::max_value() as usize);
        assert!(key.is_none());
    }
}
