use super::*;

#[derive(Default)]
pub(super) struct Enums {
    values: BTreeMap<(DefinitionId, StringId), DirectiveSiteId>,
}

impl Subgraphs {
    pub(crate) fn push_enum_value(&mut self, enum_id: DefinitionId, enum_value: StringId, directives: DirectiveSiteId) {
        self.enums.values.insert((enum_id, enum_value), directives);
    }
}

pub(crate) struct EnumValue {
    pub(crate) parent_enum_id: DefinitionId,
    pub(crate) name: StringId,
    pub(crate) directives: DirectiveSiteId,
}

impl DefinitionId {
    pub(crate) fn enum_value_by_name(self, subgraphs: &Subgraphs, name: StringId) -> Option<EnumValue> {
        subgraphs.enums.values.get(&(self, name)).map(|directives| EnumValue {
            parent_enum_id: self,
            name,
            directives: *directives,
        })
    }

    pub(crate) fn enum_values(self, subgraphs: &Subgraphs) -> impl Iterator<Item = EnumValue> {
        subgraphs
            .enums
            .values
            .range((self, StringId::MIN)..(self, StringId::MAX))
            .map(|((enum_id, value_name), directives)| EnumValue {
                parent_enum_id: *enum_id,
                name: *value_name,
                directives: *directives,
            })
    }
}
