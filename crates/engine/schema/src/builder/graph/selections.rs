use std::collections::{BTreeMap, btree_map::Entry};

use id_newtypes::IdRange;

use crate::{
    InputValueDefinitionId, InputValueInjection, InputValueInjectionId, SchemaFieldId, SchemaFieldRecord, Selections,
    ValueInjection, ValueInjectionId,
};

#[derive(Default)]
pub(crate) struct SelectionsBuilder {
    deduplicated_fields: BTreeMap<SchemaFieldRecord, SchemaFieldId>,
    pub inner: Selections,
}

impl std::ops::Deref for SelectionsBuilder {
    type Target = Selections;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for SelectionsBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SelectionsBuilder {
    // Deduplicating arguments allows us to cheaply merge field sets at runtime
    pub(crate) fn insert_field(&mut self, field: SchemaFieldRecord) -> SchemaFieldId {
        match self.deduplicated_fields.entry(field) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                self.inner.fields.push(entry.key().clone());
                *entry.insert((self.inner.fields.len() - 1).into())
            }
        }
    }

    pub(crate) fn current_injection_state(&self) -> [usize; 2] {
        [self.inner.mapping.len(), self.inner.injections.len()]
    }

    pub(crate) fn reset_injection_state(&mut self, state: [usize; 2]) {
        self.inner.mapping.truncate(state[0]);
        self.inner.injections.truncate(state[1]);
    }

    pub(crate) fn push_value_injection(&mut self, field: ValueInjection) -> ValueInjectionId {
        let id = self.inner.mapping.len().into();
        self.inner.mapping.push(field);
        id
    }

    pub(crate) fn push_input_value_injections(
        &mut self,
        input_values: &mut Vec<InputValueInjection>,
    ) -> IdRange<InputValueInjectionId> {
        let start = self.inner.injections.len();
        let end = start + input_values.len();
        input_values.sort_unstable_by_key(order_key);
        self.inner.injections.append(input_values);
        IdRange::from(start..end)
    }
}

fn order_key(inj: &InputValueInjection) -> (Option<SchemaFieldId>, InputValueDefinitionId) {
    match inj.injection {
        ValueInjection::Select { field_id, .. } => (Some(field_id), inj.definition_id),
        _ => (None, inj.definition_id),
    }
}
