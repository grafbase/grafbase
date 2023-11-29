use super::*;

#[derive(Default, Debug)]
pub(super) struct Enums {
    values: BTreeSet<(DefinitionId, StringId)>,
    // (enum name, enum value) -> deprecation
    deprecated: HashMap<(StringId, StringId), Deprecation>,
}

impl Subgraphs {
    pub(crate) fn get_enum_value_deprecation(&self, path: (StringId, StringId)) -> Option<Option<StringWalker<'_>>> {
        self.enums
            .deprecated
            .get(&path)
            .map(|deprecation| deprecation.reason.map(|reason| self.walk(reason)))
    }

    pub(crate) fn push_enum_value(&mut self, enum_id: DefinitionId, enum_value: StringId) {
        self.enums.values.insert((enum_id, enum_value));
    }

    pub(crate) fn deprecate_enum_value(&mut self, path: (StringId, StringId), reason: Deprecation) {
        self.enums.deprecated.insert(path, reason);
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
