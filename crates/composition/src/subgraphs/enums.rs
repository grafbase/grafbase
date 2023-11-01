use super::*;

#[derive(Default)]
pub(super) struct Enums {
    values: BTreeSet<(DefinitionId, StringId)>,
}

impl Subgraphs {
    pub(crate) fn push_enum_value(&mut self, enum_id: DefinitionId, enum_value: StringId) {
        self.enums.values.insert((enum_id, enum_value));
    }
}

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn enum_values(self) -> impl Iterator<Item = StringId> + 'a {
        let id = self.id;
        self.subgraphs
            .enums
            .values
            .range((id, StringId::MIN)..(id, StringId::MAX))
            .map(|(_, value)| *value)
    }
}
