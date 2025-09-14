mod buffer;

use core::f32;

use buffer::*;
use engine_schema::{DirectiveSiteId, FieldDefinition, FieldDefinitionId, Schema, TypeDefinition, TypeDefinitionId};

use fxhash::{FxBuildHasher, FxHashMap, FxHashSet};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

const DECAY_FACTOR: f32 = 0.8;

pub struct PartialSdl {
    pub max_depth: u8,
    pub search_tokens: Vec<String>,
    pub max_size_for_extra_content: usize,
    pub site_ids_and_score: Vec<(DirectiveSiteId, f32)>,
}

impl PartialSdl {
    pub fn generate(self, schema: &Schema) -> String {
        SdlBuilder {
            schema,
            content: FxHashMap::with_capacity_and_hasher(self.site_ids_and_score.len(), Default::default()),
            sites_by_depth: vec![Vec::new(); self.max_depth as usize + 1],
            params: self,
        }
        .build()
    }
}

struct SdlBuilder<'a> {
    schema: &'a Schema,
    params: PartialSdl,
    content: FxHashMap<TypeDefinitionId, WriteOptions>,
    sites_by_depth: Vec<Vec<SiteTask>>,
}

#[derive(Clone)]
struct SiteTask {
    id: DirectiveSiteId,
    score: f32,
    interfaces: bool,
}

#[derive(Clone)]
struct WriteOptions {
    interfaces: bool,
    fields_subset: Option<Vec<FieldDefinitionId>>,
    score: f32,
    depth: u8,
}

impl SdlBuilder<'_> {
    fn build(mut self) -> String {
        let mut field_ids = Vec::new();
        let mut type_ids: Vec<(TypeDefinitionId, f32)> = Vec::new();
        for (site_id, score) in std::mem::take(&mut self.params.site_ids_and_score) {
            if let Some(type_def) = site_id.as_type_definition() {
                type_ids.push((type_def, score));
            } else {
                field_ids.push((site_id.as_field().unwrap(), score));
            }
        }

        self.content.reserve(type_ids.len() + field_ids.len());

        let mut root_type_ids = vec![self.schema.query().id];
        if let Some(root_type) = self.schema.mutation() {
            root_type_ids.push(root_type.id);
        }
        if let Some(root_type) = self.schema.subscription() {
            root_type_ids.push(root_type.id);
        }

        // For all the necessary fields, we ignore all other sibling fields. This lets us ignore
        // all irrelevant fields of types like Query.
        field_ids.sort_unstable_by_key(|(field_id, _)| self.schema.walk(field_id).parent_entity_id);
        for (entity_id, field_ids_and_score) in field_ids
            .into_iter()
            .chunk_by(|(field_id, _)| self.schema.walk(*field_id).parent_entity_id)
            .into_iter()
        {
            let mut field_ids = Vec::new();
            for (field_id, score) in field_ids_and_score {
                field_ids.push(field_id);
                self.generate_content_for_field(self.schema.walk(field_id), 0, score);
            }

            // Root types will have way too many irrelevant fields.
            let fields_subset = if entity_id
                .as_object()
                .map(|id| root_type_ids.contains(&id))
                .unwrap_or_default()
            {
                Some(field_ids)
            } else {
                None
            };
            self.content.insert(
                entity_id.into(),
                WriteOptions {
                    interfaces: false,
                    fields_subset,
                    score: 0.0,
                    depth: 0,
                },
            );
        }

        for (type_id, score) in type_ids {
            self.generate_content_for_type(self.schema.walk(type_id), 0, score, false);
        }

        // This allow us to do a kind breadth first traversal of the types by iterating on those with
        // lowest depth first. This produces better scores for types used in different path and
        // thus are likely most relevant.
        while let Some((depth, types)) = self
            .sites_by_depth
            .iter_mut()
            .enumerate()
            .find(|(_, tasks)| !tasks.is_empty())
        {
            let depth = depth as u8;
            for SiteTask { id, score, interfaces } in std::mem::take(types) {
                match id.as_type_definition() {
                    Some(id) => {
                        let ty = self.schema.walk(id);
                        self.generate_content_for_type(
                            ty,
                            depth,
                            self.params.compute_initial_score(ty.name(), depth, score),
                            interfaces,
                        );
                    }
                    _ => {
                        let field = self.schema.walk(id.as_field().unwrap());
                        self.generate_content_for_field(
                            field,
                            depth,
                            self.params.compute_initial_score(field.name(), depth, score),
                        );
                    }
                }
            }
        }

        self.finalize()
    }

    fn generate_content_for_type(&mut self, ty: TypeDefinition<'_>, depth: u8, score: f32, interfaces: bool) {
        self.content
            .entry(ty.id())
            .and_modify(|entry| {
                entry.score += score;
                entry.depth = entry.depth.min(depth);
                entry.interfaces |= interfaces;
            })
            .or_insert(WriteOptions {
                interfaces,
                fields_subset: None,
                score,
                depth,
            });

        // If we're an union or interfaces we should provide their possible types if relevant as
        // they're likely to hold relevant fields. Contrary to fields we consider to be at depth +
        // 1 as they're not always necessary.
        if let Some(ty) = ty.as_composite_type() {
            let depth = depth + 1;
            let score = score * DECAY_FACTOR;
            if !ty.is_object() && depth <= self.params.max_depth && score > f32::EPSILON {
                let interfaces = ty.is_interface();
                for id in ty.possible_type_ids() {
                    self.sites_by_depth[depth as usize].push(SiteTask {
                        id: (*id).into(),
                        score,
                        interfaces,
                    });
                }
            }
        }

        // If we have fields, we need to include their arguments types and maybe their type
        // definitions.
        if let Some(entity) = ty.as_entity()
            && self.content[&ty.id()].fields_subset.is_none()
        {
            for field in entity.fields() {
                self.sites_by_depth[depth as usize].push(SiteTask {
                    id: field.id.into(),
                    score,
                    interfaces: false,
                });
            }
        }
    }

    fn generate_content_for_field(&mut self, field: FieldDefinition<'_>, depth: u8, score: f32) {
        let depth = depth + 1;
        if depth > self.params.max_depth || score <= f32::EPSILON {
            return;
        }

        for arg in field.arguments() {
            self.generate_content_for_type(arg.ty().definition(), depth, score * DECAY_FACTOR, false);
        }

        match field.ty().definition_id {
            TypeDefinitionId::Enum(_) | TypeDefinitionId::InputObject(_) => (),
            TypeDefinitionId::Interface(_)
            | TypeDefinitionId::Object(_)
            | TypeDefinitionId::Scalar(_)
            | TypeDefinitionId::Union(_) => {
                self.generate_content_for_type(field.ty().definition(), depth, score * DECAY_FACTOR, false);
            }
        }
    }

    fn finalize(self) -> String {
        let Self {
            schema,
            content,
            params,
            ..
        } = self;

        let mut buffer = Buffer::new(schema);

        let mut seen = FxHashSet::with_capacity_and_hasher(content.len(), Default::default());
        let mut queue = PriorityQueue::with_hasher(FxBuildHasher::default());

        for (type_id, opt) in content.iter() {
            tracing::debug!("{} {}|{}", self.schema.walk(type_id).name(), opt.score, opt.depth);
            if opt.depth == 0 {
                queue.push(*type_id, (OrderedFloat(f32::INFINITY), *type_id));
                seen.insert(*type_id);
            }
        }

        // We start from the "root", ie depth == 0, elements and then continue adding types based
        // on their score until we exceeded our size limit.
        while let Some((type_id, _)) = queue.pop() {
            let opt = &content[&type_id];
            if !type_id.is_scalar() && opt.depth > 0 && buffer.len() >= params.max_size_for_extra_content {
                break;
            }
            buffer.write_type_definition(schema.walk(type_id), opt);

            if let Some(entity_id) = type_id.as_entity() {
                for field in schema.walk(entity_id).fields() {
                    let ty = field.ty().definition_id;
                    if let Some(opt) = content.get(&ty)
                        && seen.insert(ty)
                    {
                        // We always include scalars, the agent cannot guess what they are
                        // without their description.
                        let score = if ty.is_scalar() { f32::INFINITY } else { opt.score };
                        queue.push(ty, (OrderedFloat(score), ty));
                    }
                    for arg in field.arguments() {
                        let ty = arg.ty().definition_id;
                        if let Some(opt) = content.get(&ty)
                            && seen.insert(ty)
                        {
                            // We always include scalars, the agent cannot guess what they are
                            // without their description.
                            let score = if ty.is_scalar() { f32::INFINITY } else { opt.score };
                            queue.push(ty, (OrderedFloat(score), ty));
                        }
                    }
                }
            }

            if let Some(composite_id) = type_id.as_composite_type().filter(|ty| !ty.is_object()) {
                for id in schema.walk(composite_id).possible_type_ids() {
                    let ty = (*id).into();
                    if let Some(opt) = content.get(&ty)
                        && seen.insert(ty)
                    {
                        queue.push(ty, (OrderedFloat(opt.score), ty));
                    }
                }
            }
        }

        buffer.into_string()
    }
}

impl PartialSdl {
    fn compute_initial_score(&self, name: &str, depth: u8, score: f32) -> f32 {
        // We favor any element that contains one of the search tokens in their name.
        let name = name.to_lowercase();
        let extra = (depth > 0) && self.search_tokens.iter().any(|token| name.contains(token));
        if extra { score / DECAY_FACTOR } else { score }
    }
}
