use std::{borrow::Cow, collections::VecDeque, marker::PhantomData, sync::Arc};

use engine::Schema;
use engine_schema::FieldDefinitionId;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tantivy::{
    Index, IndexReader, TantivyDocument, Term,
    query::{BoostQuery, Query, TermQuery},
    schema::{Field, IndexRecordOption},
};
use tokio_stream::{StreamExt as _, wrappers::WatchStream};

use super::{IntrospectTool, Tool, introspect};
use crate::EngineWatcher;

const TOP_DOCS_LIMIT: usize = 5;

pub struct SearchTool<R: engine::Runtime> {
    schema_index: tokio::sync::watch::Receiver<Arc<SchemaIndex>>,
    _marker: PhantomData<R>,
}

impl<R: engine::Runtime> Tool for SearchTool<R> {
    type Parameters = SearchParameters;

    fn name() -> &'static str {
        "search"
    }

    fn description(&self) -> Cow<'_, str> {
        format!("Search for relevant fields to use in a GraphQL query. Each matching GraphQL field will have all of its ancestor fields up to a root type. Ancestors are provided in depth order, so the first one is a field a on the root type. Always use `{}` tool afterwards to get more informations on types if you need additional fields.", IntrospectTool::<R>::name()).into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let fields = self.schema_index.borrow().clone().search(&parameters.keywords)?;
        Ok(CallToolResult {
            content: vec![rmcp::model::Content::json(fields)?],
            is_error: Some(false),
        })
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchParameters {
    keywords: Vec<String>,
}

#[derive(Serialize)]
struct FieldMatch {
    score: f32,
    field: introspect::Field,
    r#type: introspect::Type,
    root_type: &'static str,
    ancestors: Vec<introspect::Field>,
}

impl<R: engine::Runtime> SearchTool<R> {
    pub fn new(engine: &EngineWatcher<R>, enable_mutations: bool) -> anyhow::Result<Self> {
        let schema_index = Arc::new(SchemaIndex::new(engine.borrow().schema.clone(), enable_mutations)?);
        let current_hash = schema_index.schema.hash;
        let (tx, rx) = tokio::sync::watch::channel(schema_index.clone());
        let stream = WatchStream::from_changes(engine.clone());
        tokio::spawn(async move {
            let mut current_hash = current_hash;
            let mut stream = stream;
            while let Some(engine) = stream.next().await {
                if engine.schema.hash == current_hash {
                    continue;
                }
                let schema_index = SchemaIndex::new(engine.schema.clone(), enable_mutations).unwrap();
                current_hash = schema_index.schema.hash;
                tx.send(Arc::new(schema_index)).unwrap();
            }
        });
        Ok(Self {
            schema_index: rx,
            _marker: PhantomData,
        })
    }
}

struct SchemaIndex {
    schema: Arc<Schema>,
    fields: Fields,
    reader: IndexReader,
    shortest_path_parent: Vec<FieldDefinitionId>,
}

struct Fields {
    name: Field,
    type_name: Field,
    depth: Field,
    definition_id: Field,
}

impl SchemaIndex {
    fn new(schema: Arc<Schema>, enable_mutations: bool) -> anyhow::Result<Self> {
        tracing::debug!("Generating MCP schema search index");
        let start = std::time::Instant::now();

        let (shortest_path_parent, shortest_path_depth) = {
            // By default the current shortest path is oneself.
            let mut shortest_path_parent: Vec<FieldDefinitionId> =
                schema.field_definitions().map(|field| field.id).collect();
            let mut shortest_path_depth: Vec<u16> = vec![u16::MAX; schema.field_definitions().len()];

            let mut visited_objects = fixedbitset::FixedBitSet::with_capacity(schema.object_definitions().len());
            let mut visited_interfaces = fixedbitset::FixedBitSet::with_capacity(schema.interface_definitions().len());
            let mut queue = VecDeque::new();
            visited_objects.put(usize::from(schema.query().id));
            for field in schema.query().fields() {
                queue.push_back(field);
                shortest_path_depth[usize::from(field.id)] = 0;
            }
            if let Some(mutation) = schema.mutation().filter(|_| enable_mutations) {
                visited_objects.put(usize::from(mutation.id));
                for field in mutation.fields() {
                    queue.push_back(field);
                    shortest_path_depth[usize::from(field.id)] = 0;
                }
            }
            if let Some(subscription) = schema.subscription().filter(|_| enable_mutations) {
                visited_objects.put(usize::from(subscription.id));
            }
            while let Some(parent_field) = queue.pop_front() {
                let Some(entity) = parent_field.ty().definition().as_entity() else {
                    continue;
                };
                match entity {
                    engine_schema::EntityDefinition::Interface(inf) => {
                        if visited_interfaces.put(usize::from(inf.id)) {
                            continue;
                        }
                    }
                    engine_schema::EntityDefinition::Object(obj) => {
                        if visited_objects.put(usize::from(obj.id)) {
                            continue;
                        }
                    }
                }
                for field in entity.fields() {
                    shortest_path_parent[usize::from(field.id)] = parent_field.id;
                    shortest_path_depth[usize::from(field.id)] = shortest_path_depth[usize::from(parent_field.id)] + 1;
                    queue.push_back(field);
                }
            }
            (shortest_path_parent, shortest_path_depth)
        };

        let (index, fields) = {
            let mut tantivy_schema = tantivy::schema::Schema::builder();
            let fields = Fields {
                name: tantivy_schema.add_text_field("name", tantivy::schema::TEXT),
                type_name: tantivy_schema.add_text_field("type_name", tantivy::schema::TEXT),
                definition_id: tantivy_schema.add_u64_field("definition_id", tantivy::schema::STORED),
                depth: tantivy_schema.add_u64_field("depth", tantivy::schema::FAST),
            };

            let tantivy_schema = tantivy_schema.build();
            let index = Index::create_in_ram(tantivy_schema);
            let mut index_writer = index.writer(50_000_000)?;

            // Index all fields from all types
            for def in schema.field_definitions() {
                let depth = shortest_path_depth[usize::from(def.id)];
                // If mutations are disabled, some fields won't be accessible.
                if depth < u16::MAX {
                    index_writer.add_document(tantivy::doc!(
                        fields.name => def.name().to_lowercase(),
                        fields.type_name => def.ty().definition().name().to_lowercase(),
                        fields.definition_id => u32::from(def.id) as u64,
                        fields.depth => depth as u64
                    ))?;
                }
            }

            index_writer.commit()?;
            anyhow::Result::<_>::Ok((index, fields))
        }?;

        tracing::debug!("Generated search index in {:?}", start.elapsed());
        Ok(Self {
            schema,
            reader: index.reader_builder().try_into()?,
            fields,
            shortest_path_parent,
        })
    }

    fn search(&self, keywords: &[String]) -> anyhow::Result<Vec<FieldMatch>> {
        use tantivy::{
            collector::TopDocs,
            query::{BooleanQuery, FuzzyTermQuery, Occur},
            schema::Value,
        };

        tracing::debug!("Creating query for: {:?}", keywords);
        let searcher = self.reader.searcher();

        // Build a compound query that combines fuzzy searches for each keyword
        let mut subqueries = Vec::new();
        for keyword in keywords {
            let keyword = keyword.to_lowercase();
            // For each keyword, create a fuzzy query for both name and type_name fields
            let name_term = Term::from_field_text(self.fields.name, &keyword);
            let type_term = Term::from_field_text(self.fields.type_name, &keyword);

            let typos = if keyword.len() > 8 {
                2
            } else if keyword.len() > 4 {
                1
            } else {
                0
            };

            if typos > 0 {
                // Create fuzzy queries with max distance based on keyword length
                let name_query = Box::new(FuzzyTermQuery::new(name_term.clone(), typos, true));
                let type_query = Box::new(FuzzyTermQuery::new(type_term.clone(), typos, true));
                subqueries.push((Occur::Should, name_query as Box<dyn Query>));
                subqueries.push((Occur::Should, type_query as Box<dyn Query>));

                // Boosting exact matches
                let name_query = Box::new(BoostQuery::new(
                    Box::new(TermQuery::new(name_term, IndexRecordOption::Basic)),
                    1.5,
                ));
                let type_query = Box::new(BoostQuery::new(
                    Box::new(TermQuery::new(type_term, IndexRecordOption::Basic)),
                    1.5,
                ));
                subqueries.push((Occur::Should, name_query as Box<dyn Query>));
                subqueries.push((Occur::Should, type_query as Box<dyn Query>));
            } else {
                // If no typos are allowed, create exact match queries
                let name_query = Box::new(TermQuery::new(name_term, IndexRecordOption::Basic));
                let type_query = Box::new(TermQuery::new(type_term, IndexRecordOption::Basic));
                subqueries.push((Occur::Should, name_query as Box<dyn Query>));
                subqueries.push((Occur::Should, type_query as Box<dyn Query>));
            }
        }

        let query = BooleanQuery::new(subqueries);

        tracing::debug!("Searching...");
        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(TOP_DOCS_LIMIT).tweak_score(move |segment_reader: &tantivy::SegmentReader| {
                let depth_reader = segment_reader.fast_fields().u64("depth").unwrap();

                move |doc: tantivy::DocId, original_score: f32| {
                    let depth = depth_reader.first(doc).unwrap_or(256) as f32;
                    // Boost score based on inverse of depth (shallower = higher score)
                    original_score / (1.0 + depth)
                }
            }),
        )?;

        tracing::debug!("Generate response");
        let mut matches = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let definition_id = FieldDefinitionId::from(
                doc.get_first(self.fields.definition_id)
                    .and_then(|value| value.as_u64())
                    .unwrap() as u32,
            );
            let mut definition = self.schema.walk(definition_id);
            let field = definition.into();
            let r#type = definition.ty().definition().into();
            let mut ancestors = Vec::new();

            loop {
                let parent_definition_id = self.shortest_path_parent[usize::from(definition.id)];
                if parent_definition_id == definition.id {
                    break;
                }
                definition = self.schema.walk(parent_definition_id);
                ancestors.push(definition.into());
                if ancestors.len() > 10 {
                    tracing::error!("Exceed ancestors limit:\n{ancestors:#?}");
                    break;
                }
            }
            ancestors.reverse();

            matches.push(FieldMatch {
                score,
                field,
                r#type,
                ancestors,
                root_type: match definition.parent_entity_id.as_object().unwrap() {
                    id if id == self.schema.query().id => "Query",
                    id if self.schema.mutation().filter(|m| m.id == id).is_some() => "Mutation",
                    id if self.schema.subscription().filter(|s| s.id == id).is_some() => "Subscription",
                    _ => {
                        unreachable!()
                    }
                },
            });
        }

        tracing::debug!("generated all matches");
        Ok(matches)
    }
}
