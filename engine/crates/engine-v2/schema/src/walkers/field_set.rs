use crate::{FieldId, FieldSet, FieldSetItem, FieldWalker, SchemaWalker};

pub type FieldSetWalker<'a> = SchemaWalker<'a, &'a FieldSet>;
pub type FieldSetItemWalker<'a> = SchemaWalker<'a, &'a FieldSetItem>;

impl<'a> FieldSetWalker<'a> {
    pub fn is_emtpy(&self) -> bool {
        self.item.is_empty()
    }

    pub fn items(&self) -> impl Iterator<Item = FieldSetItemWalker<'a>> + 'a {
        let walker = self.walk(());
        self.item.iter().map(move |item| walker.walk(item))
    }
}

impl<'a> FieldSetItemWalker<'a> {
    pub fn field_id(&self) -> FieldId {
        self.item.field_id
    }

    pub fn field(&self) -> FieldWalker<'a> {
        self.walk(self.item.field_id)
    }

    pub fn subselection(&self) -> FieldSetWalker<'a> {
        self.walk(&self.item.subselection)
    }
}

impl<'a> From<FieldSetItemWalker<'a>> for FieldSetItem {
    fn from(walker: FieldSetItemWalker<'a>) -> Self {
        walker.item.clone()
    }
}

impl<'a> std::fmt::Debug for FieldSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FieldSet")
            .field(&self.items().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for FieldSetItemWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.item.subselection.is_empty() {
            f.debug_struct("FieldSetItem")
                .field("name", &self.field().name())
                .field("selection_set", &self.subselection())
                .finish()
        } else {
            self.field().name().fmt(f)
        }
    }
}
