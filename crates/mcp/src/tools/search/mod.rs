mod tokenizer;

use std::{borrow::Cow, collections::VecDeque, marker::PhantomData, sync::Arc};

use engine::Schema;
use engine_schema::FieldDefinitionId;
use fxhash::FxHashMap;
use rmcp::model::CallToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use tantivy::{
    Index, IndexReader, TantivyDocument, Term,
    query::{BoostQuery, DisjunctionMaxQuery, PhraseQuery, Query, TermQuery},
    schema::{Field, IndexRecordOption, TextFieldIndexing, TextOptions},
};
use tokio_stream::{StreamExt as _, wrappers::WatchStream};

use super::{IntrospectTool, SdlAndErrors, Tool};
use crate::{EngineWatcher, tools::sdl::PartialSdl};

const TOP_DOCS_LIMIT: usize = 5;
const BOUNDARIES: &[convert_case::Boundary] = &[
    convert_case::Boundary::DIGIT_UPPER,
    convert_case::Boundary::LOWER_UPPER,
    convert_case::Boundary::UNDERSCORE,
    convert_case::Boundary::ACRONYM,
];

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
        format!("Search for relevant fields to use in a GraphQL query. A list of matching fields with their score is returned with partial GraphQL SDL indicating how to query them. Use `{}` tool to request additional information on children field types if necessary to refine the selection set.", IntrospectTool::<R>::name()).into()
    }

    async fn call(&self, parameters: Self::Parameters) -> anyhow::Result<CallToolResult> {
        let resp = self.schema_index.borrow().clone().search(parameters.keywords)?;
        Ok(SdlAndErrors {
            sdl: resp.sdl,
            errors: Vec::new(),
        }
        .into())
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchParameters {
    keywords: Vec<String>,
}

pub struct SearchResponse {
    #[allow(unused)]
    matches: Vec<FieldMatch>,
    sdl: String,
}

#[allow(unused)]
struct FieldMatch {
    score: f32,
    definition_id: FieldDefinitionId,
}

impl<R: engine::Runtime> SearchTool<R> {
    pub fn new(engine: &EngineWatcher<R>, include_mutations: bool) -> anyhow::Result<Self> {
        let schema_index = Arc::new(SchemaIndex::new(engine.borrow().schema.clone(), include_mutations)?);
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
                let schema_index = SchemaIndex::new(engine.schema.clone(), include_mutations).unwrap();
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
    description: Field,
    depth: Field,
    definition_id: Field,
}

impl SchemaIndex {
    fn new(schema: Arc<Schema>, include_mutations: bool) -> anyhow::Result<Self> {
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
            if let Some(mutation) = schema.mutation() {
                visited_objects.put(usize::from(mutation.id));
                if include_mutations {
                    for field in mutation.fields() {
                        queue.push_back(field);
                        shortest_path_depth[usize::from(field.id)] = 0;
                    }
                }
            }
            if let Some(subscription) = schema.subscription() {
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
                name: tantivy_schema.add_text_field("name", tantivy::schema::STRING),
                description: tantivy_schema.add_text_field(
                    "description",
                    TextOptions::default().set_indexing_options(
                        TextFieldIndexing::default()
                            .set_tokenizer(tokenizer::TOKENIZER_NAME)
                            .set_fieldnorms(true)
                            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                    ),
                ),
                definition_id: tantivy_schema.add_u64_field("definition_id", tantivy::schema::STORED),
                depth: tantivy_schema.add_u64_field("depth", tantivy::schema::FAST),
            };

            let tantivy_schema = tantivy_schema.build();
            let index = Index::create_in_ram(tantivy_schema);
            index
                .tokenizers()
                .register(tokenizer::TOKENIZER_NAME, tokenizer::analyzer());

            let mut index_writer = index.writer(50_000_000)?;

            let mut token_buffer = Vec::new();
            // Index all fields from all types
            for def in schema.field_definitions() {
                let depth = shortest_path_depth[usize::from(def.id)];
                // If mutations are disabled, some fields won't be accessible.
                if depth < u16::MAX {
                    let mut document = TantivyDocument::default();

                    token_buffer.extend(
                        convert_case::split(&def.name(), BOUNDARIES)
                            .into_iter()
                            .map(|token| token.to_lowercase()),
                    );
                    token_buffer.extend(
                        convert_case::split(&def.ty().definition().name(), BOUNDARIES)
                            .into_iter()
                            .map(|token| token.to_lowercase()),
                    );
                    token_buffer.sort_unstable();
                    token_buffer.dedup();

                    for token in token_buffer.drain(..) {
                        document.add_field_value(fields.name, token);
                    }
                    if let Some(desc) = def.description() {
                        document.add_field_value(fields.description, desc);
                    }
                    if let Some(desc) = def.ty().definition().description() {
                        document.add_field_value(fields.description, desc);
                    }

                    document.add_field_value(fields.definition_id, u32::from(def.id) as u64);
                    document.add_field_value(fields.depth, depth as u64);

                    index_writer.add_document(document)?;
                }
            }

            index_writer.commit()?;
            anyhow::Result::<_>::Ok((index, fields))
        }?;

        tracing::debug!("Generated search index took {:?}", start.elapsed());
        Ok(Self {
            schema,
            reader: index.reader_builder().try_into()?,
            fields,
            shortest_path_parent,
        })
    }

    fn search(&self, keywords: Vec<String>) -> anyhow::Result<SearchResponse> {
        use tantivy::{
            collector::TopDocs,
            query::{BooleanQuery, FuzzyTermQuery, Occur},
            schema::Value,
        };

        let start = std::time::Instant::now();
        tracing::debug!("Creating query for: {:?}", keywords);
        let searcher = self.reader.searcher();

        // Build a compound query that combines fuzzy searches for each keyword
        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let mut search_tokens = Vec::new();
        let mut analyzer = tokenizer::analyzer();
        for keyword in &keywords {
            let tokens = convert_case::split(keyword, &convert_case::Boundary::defaults());
            let mut queries: Vec<Box<dyn Query>> = Vec::new();
            for token in tokens {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                let token = token.to_lowercase();

                let typos = if token.len() > 4 { 1 } else { 0 };

                let term = Term::from_field_text(self.fields.name, &token);
                search_tokens.push(token);
                if typos > 0 {
                    queries.push(Box::new(FuzzyTermQuery::new(term.clone(), typos, true)));
                    queries.push(Box::new(BoostQuery::new(
                        Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
                        1.2,
                    )));
                } else {
                    queries.push(Box::new(TermQuery::new(term, IndexRecordOption::Basic)));
                }
            }
            subqueries.push((Occur::Should, Box::new(DisjunctionMaxQuery::new(queries))));
        }

        for keyword in &keywords {
            let mut queries: Vec<Box<dyn Query>> = Vec::new();
            let mut terms_with_offset: Vec<(usize, Term)> = Vec::new();
            analyzer.token_stream(keyword).process(&mut |token| {
                let term = Term::from_field_text(self.fields.description, &token.text);
                terms_with_offset.push((token.position, term));
            });

            for (_, term) in &terms_with_offset {
                let typos = if term.len_bytes() > 4 { 1 } else { 0 };

                if typos > 0 {
                    queries.push(Box::new(FuzzyTermQuery::new(term.clone(), typos, true)));
                    queries.push(Box::new(BoostQuery::new(
                        Box::new(TermQuery::new(term.clone(), IndexRecordOption::Basic)),
                        1.2,
                    )));
                } else {
                    queries.push(Box::new(TermQuery::new(term.clone(), IndexRecordOption::Basic)));
                }
            }

            subqueries.push((Occur::Should, Box::new(DisjunctionMaxQuery::new(queries))));
            if terms_with_offset.len() > 1 {
                subqueries.push((Occur::Should, Box::new(PhraseQuery::new_with_offset(terms_with_offset))));
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

        tracing::debug!(
            "Search took {:?} and found {} documents",
            start.elapsed(),
            top_docs.len()
        );

        let mut matches = Vec::new();
        let mut site_id_to_score = FxHashMap::default();
        for (mut score, doc_address) in top_docs {
            score += 1.0;

            let doc: TantivyDocument = searcher.doc(doc_address)?;
            let mut definition_id = FieldDefinitionId::from(
                doc.get_first(self.fields.definition_id)
                    .and_then(|value| value.as_u64())
                    .unwrap() as u32,
            );

            let definition = self.schema.walk(definition_id);
            tracing::debug!(
                "Search matched {}.{} with score {}",
                definition.parent_entity().name(),
                definition.name(),
                score
            );
            matches.push(FieldMatch { score, definition_id });
            site_id_to_score
                .entry(definition_id.into())
                .and_modify(|current| *current += score)
                .or_insert(score);
            loop {
                let parent_definition_id = self.shortest_path_parent[usize::from(definition_id)];
                if parent_definition_id == definition_id {
                    break;
                }
                definition_id = parent_definition_id;
                site_id_to_score.entry(definition_id.into()).or_insert(0.1);
            }
        }

        let sdl = PartialSdl {
            search_tokens,
            max_depth: 3,
            max_size_for_extra_content: 6144,
            site_ids_and_score: site_id_to_score.into_iter().collect(),
        }
        .generate(&self.schema);

        Ok(SearchResponse { matches, sdl })
    }
}
