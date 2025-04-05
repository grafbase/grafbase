use std::{borrow::Cow, collections::VecDeque, sync::Arc};

use engine::Schema;
use engine_schema::FieldDefinitionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tantivy::{
    Index, IndexReader, TantivyDocument, Term,
    query::{Query, TermQuery},
    schema::{Field, IndexRecordOption},
};
use tokio_stream::{StreamExt as _, wrappers::WatchStream};

use super::Tool;
use crate::EngineWatcher;

const TOP_DOCS_LIMIT: usize = 20;

pub struct SearchTool {
    schema_index: tokio::sync::watch::Receiver<Arc<SchemaIndex>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchParameters {
    keywords: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    fields: Vec<FieldMatch>,
}

#[derive(Serialize)]
struct FieldMatch {
    query_path: Vec<ShortestQueryPathSegment>,
}

#[derive(Serialize)]
struct ShortestQueryPathSegment {
    field: String,
    output_type: String,
    arguments: Vec<FieldArgument>,
}

#[derive(Serialize)]
struct FieldArgument {
    name: String,
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<serde_json::Value>,
}

impl SearchTool {
    pub fn new(engine: &EngineWatcher<impl engine::Runtime>) -> anyhow::Result<Self> {
        let schema_index = Arc::new(SchemaIndex::new(engine.borrow().schema.clone())?);
        let (tx, rx) = tokio::sync::watch::channel(schema_index.clone());
        let stream = WatchStream::from_changes(engine.clone());
        tokio::spawn(async move {
            let mut stream = stream;
            while let Some(engine) = stream.next().await {
                let schema_index = SchemaIndex::new(engine.schema.clone()).unwrap();
                tx.send(Arc::new(schema_index)).unwrap();
            }
        });
        Ok(Self { schema_index: rx })
    }
}

impl Tool for SearchTool {
    type Parameters = SearchParameters;
    type Response = SearchResponse;
    type Error = String;

    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> Cow<'_, str> {
        "Case insensisitve search for fields in the GraphQL schema by name or type name. Supports fuzzy matching for longer keywords and returns up to 20 most relevant matches, with results scored based on their depth in the schema. Each match includes its shortest possible query path, from a root type. Each segment includes the field's name, output type, and arguments with their types and default values.".into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<Result<Self::Response, Self::Error>> {
        let fields = self.schema_index.borrow().clone().search(&parameters.keywords)?;
        Ok(Ok(SearchResponse { fields }))
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
    fn new(schema: Arc<Schema>) -> anyhow::Result<Self> {
        let (shortest_path_parent, shortest_path_depth) = {
            // By default the current shortest path is oneself.
            let mut shortest_path_parent: Vec<FieldDefinitionId> =
                schema.field_definitions().map(|field| field.id).collect();
            let mut shortest_path_depth: Vec<u16> = vec![0; schema.field_definitions().len()];

            let mut visited_objects = fixedbitset::FixedBitSet::with_capacity(schema.object_definitions().len());
            let mut visited_interfaces = fixedbitset::FixedBitSet::with_capacity(schema.interface_definitions().len());
            let mut queue = VecDeque::new();
            queue.extend(schema.query().fields());
            if let Some(mutation) = schema.mutation() {
                queue.extend(mutation.fields());
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
                index_writer.add_document(tantivy::doc!(
                    fields.name => def.name().to_lowercase(),
                    fields.type_name => def.ty().definition().name().to_lowercase(),
                    fields.definition_id => u32::from(def.id) as u64,
                    fields.depth => depth as u64
                ))?;
            }

            index_writer.commit()?;
            anyhow::Result::<_>::Ok((index, fields))
        }?;

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
                let name_query = Box::new(FuzzyTermQuery::new(name_term, typos, true));
                let type_query = Box::new(FuzzyTermQuery::new(type_term, typos, true));
                // Add both queries as "should" clauses
                subqueries.push((Occur::Should, name_query as Box<dyn Query>));
                subqueries.push((Occur::Should, type_query as Box<dyn Query>));
                continue;
            } else {
                // If no typos are allowed, create exact match queries
                let name_query = Box::new(TermQuery::new(name_term, IndexRecordOption::Basic));
                let type_query = Box::new(TermQuery::new(type_term, IndexRecordOption::Basic));
                // Add both queries as "should" clauses
                subqueries.push((Occur::Should, name_query as Box<dyn Query>));
                subqueries.push((Occur::Should, type_query as Box<dyn Query>));
            }
        }

        // Combine all subqueries with BooleanQuery
        let query = BooleanQuery::new(subqueries);
        // Create a custom collector that boosts scores based on depth
        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(TOP_DOCS_LIMIT).tweak_score(move |segment_reader: &tantivy::SegmentReader| {
                let depth_reader = segment_reader.fast_fields().u64("depth").unwrap();

                move |doc: tantivy::DocId, original_score: f32| {
                    let depth = depth_reader.first(doc).unwrap_or(256) as f32;
                    // Boost score based on inverse of depth (shallower = higher score)
                    original_score * (1.0 / (1.0 + depth))
                }
            }),
        )?;

        let mut matches = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut definition_id = FieldDefinitionId::from(
                doc.get_first(self.fields.definition_id)
                    .and_then(|value| value.as_u64())
                    .unwrap() as u32,
            );

            let mut query_path = Vec::new();
            loop {
                let definition = self.schema.walk(definition_id);
                query_path.push(ShortestQueryPathSegment {
                    field: definition.name().to_owned(),
                    output_type: definition.ty().to_string(),
                    arguments: definition
                        .arguments()
                        .map(|arg| FieldArgument {
                            name: arg.name().to_owned(),
                            r#type: arg.ty().to_string(),
                            default_value: arg.default_value().map(|v| serde_json::to_value(v).unwrap()),
                        })
                        .collect(),
                });
                let parent_definition_id = self.shortest_path_parent[usize::from(definition_id)];
                if parent_definition_id == definition_id {
                    break;
                }
                definition_id = parent_definition_id;
            }
            query_path.reverse();

            matches.push(FieldMatch { query_path });
        }
        Ok(matches)
    }
}
