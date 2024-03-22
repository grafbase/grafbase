use std::cmp::Ordering;

use crate::{RequiredField, RequiredFieldSet, RequiredFieldSetArgumentsId, SchemaWalker};

pub type RequiredFieldsWalker<'a> = SchemaWalker<'a, &'a RequiredFieldSet>;
pub type RequiredFieldWalker<'a> = SchemaWalker<'a, &'a RequiredField>;

impl std::fmt::Debug for RequiredFieldsWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RequiredFields")
            .field(&self.item.iter().map(|field| self.walk(field)).collect::<Vec<_>>())
            .finish()
    }
}

impl std::fmt::Debug for RequiredFieldWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("RequiredField");
        f.field("name", &self.walk(self.item.definition_id).name());
        if let Some(arguments_id) = self.item.arguments_id {
            f.field("arguments", &self.walk(arguments_id));
        }
        if !self.item.subselection.is_empty() {
            f.field("subselection", &self.walk(&self.item.subselection));
        }
        f.finish()
    }
}

pub type RequiredFieldArgumentsWalker<'a> = SchemaWalker<'a, RequiredFieldSetArgumentsId>;

impl<'a> Ord for RequiredFieldArgumentsWalker<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let left = self.as_ref();
        let right = other.as_ref();
        left.len().cmp(&right.len()).then_with(|| {
            // arguments are sorted by InputValueDefinitionId
            for ((lid, lvalue), (rid, rvalue)) in left.iter().zip(right.iter()) {
                match lid
                    .cmp(rid)
                    .then_with(|| self.walk(&self.schema[*lvalue]).cmp(&self.walk(&self.schema[*rvalue])))
                {
                    Ordering::Equal => (),
                    other => return other,
                }
            }
            Ordering::Equal
        })
    }
}

impl PartialEq for RequiredFieldArgumentsWalker<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for RequiredFieldArgumentsWalker<'_> {}

impl PartialOrd for RequiredFieldArgumentsWalker<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for RequiredFieldArgumentsWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("RequiredFieldArguments");
        for (input_value_definition_id, value_id) in self.as_ref().iter() {
            f.field(
                self.walk(*input_value_definition_id).name(),
                &self.walk(&self.schema[*value_id]),
            );
        }
        f.finish()
    }
}
