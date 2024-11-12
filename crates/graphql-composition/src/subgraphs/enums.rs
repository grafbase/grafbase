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

impl<'a> DefinitionWalker<'a> {
    pub(crate) fn enum_value_by_name(self, name: StringId) -> Option<EnumValueWalker<'a>> {
        self.subgraphs
            .enums
            .values
            .get(&(self.id, name))
            .map(|directives| EnumValueWalker {
                id: (self.id, name, *directives),
                subgraphs: self.subgraphs,
            })
    }

    pub(crate) fn enum_values(self) -> impl Iterator<Item = EnumValueWalker<'a>> + 'a {
        let id = self.id;
        self.subgraphs
            .enums
            .values
            .range((id, StringId::MIN)..(id, StringId::MAX))
            .map(|((enum_id, value_name), directives)| EnumValueWalker {
                id: (*enum_id, *value_name, *directives),
                subgraphs: self.subgraphs,
            })
    }
}

pub(crate) type EnumValueWalker<'a> = Walker<'a, (DefinitionId, StringId, DirectiveSiteId)>;

impl<'a> EnumValueWalker<'a> {
    pub(crate) fn name(self) -> StringWalker<'a> {
        let (_enum_id, value_name, _directives) = self.id;
        self.walk(value_name)
    }

    pub(crate) fn directives(self) -> DirectiveSiteWalker<'a> {
        let (_enum_id, _value_name, directives) = self.id;
        self.walk(directives)
    }
}
