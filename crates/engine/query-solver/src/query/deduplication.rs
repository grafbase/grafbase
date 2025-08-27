use hashbrown::{HashTable, hash_table::Entry};
use operation::OperationContext;
use petgraph::visit::GraphBase;
use rapidhash::fast::SeedableState;
use schema::ResolverDefinitionId;
use std::hash::{BuildHasher as _, Hash, Hasher};
use walker::Walk as _;

use crate::{
    DeduplicationId, Query, QueryField, QueryFieldId, QueryOrSchemaSortedFieldArgumentIds, are_arguments_equivalent,
};

pub(crate) struct DeduplicationMap {
    table: HashTable<DeduplicatedEntry>,
    hash_seed: SeedableState<'static>,
}

struct DeduplicatedEntry {
    hash: u64,
    id: DeduplicationId,
    record: Record,
}

#[derive(strum::EnumDiscriminants, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
enum Record {
    Field(QueryFieldId),
    Resolver(schema::ResolverDefinitionId),
}

impl<G: GraphBase> Query<G, crate::steps::SolutionSpace> {
    pub fn get_or_insert_field_deduplication_id(
        &mut self,
        ctx: OperationContext<'_>,
        id: QueryFieldId,
    ) -> DeduplicationId {
        self.step.deduplication_map.get_or_insert_field(ctx, &self.fields, id)
    }
}

impl DeduplicationMap {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            table: HashTable::with_capacity(size),
            hash_seed: Default::default(),
        }
    }

    pub fn get_or_insert_resolver(&mut self, id: ResolverDefinitionId) -> DeduplicationId {
        let hash = {
            let mut hasher = self.hash_seed.build_hasher();
            RecordDiscriminants::Resolver.hash(&mut hasher);
            id.hash(&mut hasher);
            hasher.finish()
        };

        let n = self.table.len();
        match self.table.entry(
            hash,
            |entry| match entry.record {
                Record::Resolver(existing) => existing == id,
                _ => false,
            },
            |entry| entry.hash,
        ) {
            Entry::Occupied(entry) => entry.get().id,
            Entry::Vacant(entry) => {
                let dedup_id = DeduplicationId::from(u16::try_from(n).expect("Too many entrys to deduplicate."));
                entry.insert(DeduplicatedEntry {
                    hash,
                    id: dedup_id,
                    record: Record::Resolver(id),
                });
                dedup_id
            }
        }
    }

    pub fn get_or_insert_field<'op>(
        &mut self,
        ctx: OperationContext<'op>,
        fields: &[QueryField],
        id: QueryFieldId,
    ) -> DeduplicationId {
        // Maybe we should avoid keeping this data in the table and have an id instead?
        let field = &fields[usize::from(id)];
        let hash = {
            let mut hasher = self.hash_seed.build_hasher();
            RecordDiscriminants::Field.hash(&mut hasher);
            field.type_conditions.hash(&mut hasher);
            field.flat_directive_id.hash(&mut hasher);
            field.response_key.hash(&mut hasher);
            field.definition_id.hash(&mut hasher);
            field.sorted_argument_ids.len().hash(&mut hasher);
            match field.sorted_argument_ids {
                QueryOrSchemaSortedFieldArgumentIds::Query(ids) => {
                    for arg in ids.walk(ctx) {
                        arg.definition_id.hash(&mut hasher);
                    }
                }
                QueryOrSchemaSortedFieldArgumentIds::Schema(ids) => {
                    for arg in ids.walk(ctx) {
                        arg.definition_id.hash(&mut hasher);
                    }
                }
            }
            hasher.finish()
        };

        let n = self.table.len();
        match self.table.entry(
            hash,
            |entry| match entry.record {
                Record::Field(id) => {
                    let existing = &fields[usize::from(id)];
                    ((existing.type_conditions == field.type_conditions)
                        & (existing.response_key == field.response_key)
                        & (existing.definition_id == field.definition_id)
                        & (existing.flat_directive_id == field.flat_directive_id))
                        && are_arguments_equivalent(ctx, existing.sorted_argument_ids, field.sorted_argument_ids)
                }
                _ => false,
            },
            |entry| entry.hash,
        ) {
            Entry::Occupied(entry) => entry.get().id,
            Entry::Vacant(entry) => {
                let dedup_id = DeduplicationId::from(u16::try_from(n).expect("Too many entrys to deduplicate."));
                entry.insert(DeduplicatedEntry {
                    hash,
                    id: dedup_id,
                    record: Record::Field(id),
                });
                dedup_id
            }
        }
    }
}
