use super::{Directive, EnumDefinitionId, EnumValueId, FederatedGraph, StringId, View, ViewNested};

pub type EnumValue<'a> = ViewNested<'a, EnumValueId, EnumValueRecord>;

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumValueDefinition")
            .field("enum", &self.then(|v| v.enum_id).then(|enm| enm.name).as_str())
            .field("value", &self.then(|v| v.value).as_str())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct EnumValueRecord {
    pub enum_id: EnumDefinitionId,
    pub value: StringId,
    pub directives: Vec<Directive>,
    pub description: Option<StringId>,
}

impl FederatedGraph {
    pub fn enum_value_range(&self, enum_id: EnumDefinitionId) -> (EnumValueId, usize) {
        let mut values = self.iter_enum_values(enum_id);
        let Some(start) = values.next() else {
            return (EnumValueId::from(0), 0);
        };

        (start.id(), values.count() + 1)
    }

    pub fn find_enum_value_by_name(&self, enum_id: EnumDefinitionId, name: &str) -> Option<EnumValue<'_>> {
        self.iter_enum_values(enum_id).find(|value| self[value.value] == name)
    }

    pub fn find_enum_value_by_name_id(&self, enum_id: EnumDefinitionId, name_id: StringId) -> Option<EnumValue<'_>> {
        self.iter_enum_values(enum_id).find(|value| value.value == name_id)
    }

    pub fn iter_enum_values(&self, enum_id: EnumDefinitionId) -> impl Iterator<Item = EnumValue<'_>> + Clone {
        let start = self.enum_values.partition_point(|value| value.enum_id < enum_id);
        self.enum_values[start..]
            .iter()
            .enumerate()
            .take_while(move |(_idx, value)| value.enum_id == enum_id)
            .map(move |(idx, value)| ViewNested {
                view: View {
                    id: EnumValueId::from(start + idx),
                    record: value,
                },
                graph: self,
            })
    }

    pub fn push_enum_value(&mut self, enum_value: EnumValueRecord) -> EnumValueId {
        let id = self.enum_values.len().into();
        self.enum_values.push(enum_value);
        id
    }
}
