use std::fmt::Write;

use itertools::Itertools;
use schema::{EntityDefinitionId, FieldDefinitionId, Schema};
use walker::Walk;

use crate::Operation;

pub struct UsedFields<'a> {
    schema: &'a Schema,
    fields: Vec<(EntityDefinitionId, FieldDefinitionId)>,
}

pub(super) fn compute<'s>(schema: &'s Schema, operation: &Operation) -> UsedFields<'s> {
    let mut fields = Vec::with_capacity(operation.data_fields.len());
    let introspection = &schema.subgraphs.introspection;
    for field in &operation.data_fields {
        let field = field.definition_id.walk(schema);
        if let EntityDefinitionId::Object(object_id) = field.parent_entity_id {
            // Skipping introspection related fields
            if introspection.meta_fields.contains(&field.id) || introspection.meta_objects.contains(&object_id) {
                continue;
            }
        }
        fields.push((field.parent_entity_id, field.id))
    }
    fields.sort_unstable();

    UsedFields { schema, fields }
}

impl UsedFields<'_> {
    // Didn't find a better way to define the initial string capacity to something sensible.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        let mut out = String::with_capacity(self.fields.len() * 4);
        write!(out, "{self}").unwrap();
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
    fields: std::vec::IntoIter<(EntityDefinitionId, FieldDefinitionId)>,
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
