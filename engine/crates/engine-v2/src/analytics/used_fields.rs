use std::fmt::Write;

use itertools::Itertools;
use schema::{EntityId, FieldDefinitionId, Schema};

use crate::operation::Operation;

pub struct UsedFields<'a> {
    schema: &'a Schema,
    fields: Vec<(EntityId, FieldDefinitionId)>,
}

pub(super) fn compute<'s>(schema: &'s Schema, operation: &Operation) -> UsedFields<'s> {
    let mut fields = Vec::with_capacity(operation.fields.len());
    for field in &operation.fields {
        let Some(definition_id) = field.definition_id() else {
            continue;
        };

        let field = schema.walk(definition_id);
        let entity = field.parent_entity();
        // Skipping introspection related fields
        if !entity.name().starts_with("__") && !field.name().starts_with("__") {
            fields.push((entity.id(), definition_id))
        }
    }
    fields.sort_unstable();

    UsedFields { schema, fields }
}

impl UsedFields<'_> {
    // Didn't find a better way to define the initial string capacity to something sensible.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(self.fields.len() * 4);
        write!(out, "{}", self).unwrap();
        out
    }
}

impl std::fmt::Display for UsedFields<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (entity_id, field_definitions) in self
            .fields
            .iter()
            .copied()
            .dedup()
            .chunk_by(|(entity_id, _)| *entity_id)
            .into_iter()
        {
            f.write_str(self.schema.walk(entity_id).name())?;
            f.write_char('.')?;
            for s in Itertools::intersperse(
                field_definitions.map(|(_, definition_id)| self.schema.walk(definition_id).name()),
                "+",
            ) {
                f.write_str(s)?;
            }
            f.write_char(',')?;
        }

        Ok(())
    }
}

impl<'a> IntoIterator for UsedFields<'a> {
    type Item = (&'a str, &'a str);
    type IntoIter = UsedFieldsIntoIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        UsedFieldsIntoIter {
            schema: self.schema,
            fields: self.fields.into_iter(),
        }
    }
}

pub struct UsedFieldsIntoIter<'a> {
    schema: &'a Schema,
    fields: std::vec::IntoIter<(EntityId, FieldDefinitionId)>,
}

impl<'a> Iterator for UsedFieldsIntoIter<'a> {
    type Item = (&'a str, &'a str);
    fn next(&mut self) -> Option<Self::Item> {
        let (entity_id, definition_id) = self.fields.next()?;
        Some((
            self.schema.walk(entity_id).name(),
            self.schema.walk(definition_id).name(),
        ))
    }
}
