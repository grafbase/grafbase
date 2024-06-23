use crate::{FieldDefinitionWalker, RequiredFieldId, RequiredFieldSet, RequiredFieldSetItem, SchemaWalker};

pub type RequiredFieldsWalker<'a> = SchemaWalker<'a, &'a RequiredFieldSet>;
pub type RequiredFieldSetItemWalker<'a> = SchemaWalker<'a, &'a RequiredFieldSetItem>;

impl std::fmt::Debug for RequiredFieldsWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RequiredFields")
            .field(&self.item.iter().map(|field| self.walk(field)).collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> RequiredFieldSetItemWalker<'a> {
    pub fn required_field_id(&self) -> RequiredFieldId {
        self.item.id
    }

    pub fn name(&self) -> &'a str {
        self.definition().name()
    }

    pub fn definition(&self) -> FieldDefinitionWalker<'a> {
        self.walk(self.schema[self.item.id].definition_id)
    }

    pub fn subselection(&self) -> impl Iterator<Item = RequiredFieldSetItemWalker<'a>> + '_ {
        self.item.subselection.iter().map(move |id| self.walk(id))
    }
}

impl std::fmt::Debug for RequiredFieldSetItemWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("RequiredField");
        f.field("name", &self.name());
        // FIXME: add arguments back in Debug.
        f.field("arguments", &"FIXME!!");
        if !self.item.subselection.is_empty() {
            f.field("subselection", &self.walk(&self.item.subselection));
        }
        f.finish()
    }
}
