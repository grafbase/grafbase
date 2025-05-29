use std::collections::{BTreeMap, btree_map::Entry};

use id_newtypes::IdRange;

use crate::{
    ArgumentInjectionId, ArgumentInjectionRecord, ArgumentValueInjection, ArgumentValueInjectionId,
    KeyValueInjectionId, KeyValueInjectionRecord, SchemaFieldId, SchemaFieldRecord, Selections, StringId,
    ValueInjection, ValueInjectionId,
};

pub(crate) struct SelectionsBuilder {
    deduplicated_fields: BTreeMap<SchemaFieldRecord, SchemaFieldId>,
    pub inner: Selections,
}

impl Default for SelectionsBuilder {
    fn default() -> Self {
        let mut builder = Self {
            deduplicated_fields: BTreeMap::new(),
            inner: Default::default(),
        };
        builder.inner.injections.push(ValueInjection::Identity);
        builder
    }
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

pub(crate) type SelectionsState = [usize; 4];

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

    pub(crate) fn current_state(&self) -> SelectionsState {
        [
            self.inner.argument_injections.len(),
            self.inner.argument_value_injections.len(),
            self.inner.injections.len(),
            self.inner.key_value_injections.len(),
        ]
    }

    pub(crate) fn reset(&mut self, state: SelectionsState) {
        self.inner.argument_injections.truncate(state[0]);
        self.inner.argument_value_injections.truncate(state[1]);
        self.inner.injections.truncate(state[2]);
        self.inner.key_value_injections.truncate(state[3]);
    }

    pub(crate) fn push_argument_value_injection(
        &mut self,
        injection: ArgumentValueInjection,
    ) -> ArgumentValueInjectionId {
        let id = self.inner.argument_value_injections.len().into();
        self.inner.argument_value_injections.push(injection);
        id
    }

    pub(crate) fn push_injection(&mut self, injection: ValueInjection) -> ValueInjectionId {
        match injection {
            ValueInjection::Identity => ValueInjectionId::from(0usize),
            injection => {
                let id = self.inner.injections.len().into();
                self.inner.injections.push(injection);
                id
            }
        }
    }

    pub(crate) fn push_injections(
        &mut self,
        injections: impl IntoIterator<Item = ValueInjection>,
    ) -> IdRange<ValueInjectionId> {
        let start = self.inner.injections.len();
        self.inner.injections.extend(injections);
        let end = self.inner.injections.len();
        self.inner.injections[start..end].sort_unstable_by_key(|inj| match inj {
            ValueInjection::Select { field_id, .. } => Some(*field_id),
            _ => None,
        });
        IdRange::from(start..end)
    }

    pub(crate) fn push_key_value_injections(
        &mut self,
        key_values: impl IntoIterator<Item = KeyValueInjectionRecord>,
    ) -> IdRange<KeyValueInjectionId> {
        let start = self.inner.key_value_injections.len();
        self.inner.key_value_injections.extend(key_values);
        let end = self.inner.key_value_injections.len();
        self.inner.key_value_injections[start..end].sort_unstable_by_key(order_key);
        IdRange::from(start..end)
    }

    pub(crate) fn push_argument_injections(
        &mut self,
        arguments: impl IntoIterator<Item = ArgumentInjectionRecord>,
    ) -> IdRange<ArgumentInjectionId> {
        let start = self.inner.argument_injections.len();
        self.inner.argument_injections.extend(arguments);
        let end = self.inner.argument_injections.len();
        IdRange::from(start..end)
    }
}

fn order_key(inj: &KeyValueInjectionRecord) -> (Option<SchemaFieldId>, StringId) {
    match inj.value {
        ValueInjection::Select { field_id, .. } => (Some(field_id), inj.key_id),
        _ => (None, inj.key_id),
    }
}
