use hashbrown::{HashTable, hash_table::Entry};
use operation::OperationContext;
use petgraph::visit::GraphBase;
use rapidhash::fast::SeedableState;
use schema::ResolverDefinitionId;
use std::{
    hash::{BuildHasher as _, Hash, Hasher},
    num::NonZero,
};

use crate::{Query, QueryFieldId, solve::DeduplicationId};

pub(in crate::solve) struct DeduplicationMap {
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
                let dedup_id =
                    DeduplicationId(NonZero::new(u16::try_from(n + 1).expect("Too many fields/resolvers")).unwrap());
                entry.insert(DeduplicatedEntry {
                    hash,
                    id: dedup_id,
                    record: Record::Resolver(id),
                });
                dedup_id
            }
        }
    }

    pub fn get_or_insert_field<'op, G: GraphBase, S>(
        &mut self,
        ctx: OperationContext<'op>,
        query: &Query<G, S>,
        id: QueryFieldId,
    ) -> DeduplicationId {
        // Maybe we should avoid keeping this data in the table and have an id instead?
        let field = &query[id];
        let hash = {
            let mut hasher = self.hash_seed.build_hasher();
            RecordDiscriminants::Field.hash(&mut hasher);
            field.equivalence_hash(query, ctx, &mut hasher);
            hasher.finish()
        };

        let n = self.table.len();
        match self.table.entry(
            hash,
            |entry| match entry.record {
                Record::Field(id) => query[id].is_equivalent(query, ctx, field),
                _ => false,
            },
            |entry| entry.hash,
        ) {
            Entry::Occupied(entry) => entry.get().id,
            Entry::Vacant(entry) => {
                let dedup_id =
                    DeduplicationId(NonZero::new(u16::try_from(n + 1).expect("Too many fields/resolvers")).unwrap());
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
