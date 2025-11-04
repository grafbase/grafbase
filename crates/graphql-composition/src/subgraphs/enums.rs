use super::*;

#[derive(Default)]
pub(super) struct Enums {
    pub(super) values: Vec<EnumValue>,
}

impl Subgraphs {
    pub(crate) fn push_enum_value(&mut self, enum_value: EnumValue) {
        self.enums.values.push(enum_value);
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct EnumValue {
    pub(crate) parent_enum_id: DefinitionId,
    pub(crate) name: StringId,
    pub(crate) description: Option<StringId>,
    pub(crate) directives: DirectiveSiteId,
}

impl DefinitionId {
    pub(crate) fn enum_value_by_name(
        self,
        subgraphs: &Subgraphs,
        name: StringId,
    ) -> Option<View<'_, EnumValueId, EnumValue>> {
        subgraphs
            .enums
            .values
            .binary_search_by_key(&(self, name), |v| (v.parent_enum_id, v.name))
            .ok()
            .map(|idx| subgraphs.at(EnumValueId::from(idx)))
    }

    pub(crate) fn enum_values(self, subgraphs: &Subgraphs) -> impl Iterator<Item = View<'_, EnumValueId, EnumValue>> {
        let start = subgraphs.enums.values.partition_point(|v| v.parent_enum_id < self);

        subgraphs.enums.values[start..]
            .iter()
            .take_while(move |v| v.parent_enum_id == self)
            .enumerate()
            .map(move |(idx, v)| View {
                id: EnumValueId::from(start + idx),
                record: v,
            })
    }
}
