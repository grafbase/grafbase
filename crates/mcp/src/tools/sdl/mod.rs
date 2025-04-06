mod buffer;

use buffer::*;
use engine::Schema;
use engine_schema::{DirectiveSiteId, FieldDefinition, FieldDefinitionId, TypeDefinition, TypeDefinitionId};

use fxhash::FxHashMap;
use itertools::Itertools;

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
            fields_by_depth: vec![Vec::new(); self.max_depth as usize + 1],
            params: self,
        }
        .build()
    }
}

struct SdlBuilder<'a> {
    schema: &'a Schema,
    params: PartialSdl,
    content: FxHashMap<TypeDefinitionId, SdlOptions>,
    fields_by_depth: Vec<Vec<(FieldDefinitionId, f32)>>,
}

struct SdlOptions {
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
            match site_id {
                DirectiveSiteId::Field(field_id) => field_ids.push((field_id, score)),
                DirectiveSiteId::Enum(id) => type_ids.push((id.into(), score)),
                DirectiveSiteId::EnumValue(id) => type_ids.push((self.schema.walk(id).parent_enum_id.into(), score)),
                DirectiveSiteId::InputObject(id) => type_ids.push((id.into(), score)),
                DirectiveSiteId::InputValue(_) => {
                    unreachable!(
                        "We don't have the necessary context for it today, nor do we generate them anywhere in MCP."
                    )
                }
                DirectiveSiteId::Interface(id) => type_ids.push((id.into(), score)),
                DirectiveSiteId::Object(id) => type_ids.push((id.into(), score)),
                DirectiveSiteId::Scalar(id) => type_ids.push((id.into(), score)),
                DirectiveSiteId::Union(id) => type_ids.push((id.into(), score)),
            }
        }

        self.content.reserve(type_ids.len() + field_ids.len());
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
            self.content.insert(
                entity_id.into(),
                SdlOptions {
                    interfaces: false,
                    fields_subset: Some(field_ids),
                    score: 0.0,
                    depth: 0,
                },
            );
        }

        for (type_id, score) in type_ids {
            self.generate_content_for_type(self.schema.walk(type_id), 0, score);
        }

        while let Some((depth, types)) = self
            .fields_by_depth
            .iter_mut()
            .enumerate()
            .find(|(_, tasks)| !tasks.is_empty())
        {
            let depth = depth as u8;
            for (field_id, score) in std::mem::take(types) {
                let field = self.schema.walk(field_id);
                self.generate_content_for_field(
                    field,
                    depth,
                    self.params.compute_initial_score(field.name(), depth, score),
                );
            }
        }

        self.finalize()
    }

    fn generate_content_for_type(&mut self, ty: TypeDefinition<'_>, depth: u8, score: f32) {
        self.content
            .entry(ty.id())
            .and_modify(|entry| {
                entry.score *= score;
                entry.depth = entry.depth.min(depth);
            })
            .or_insert(SdlOptions {
                interfaces: false,
                fields_subset: None,
                score: self.params.compute_initial_score(ty.name(), depth, score),
                depth,
            });
        if let Some(inf) = ty.as_interface() {
            for obj in inf.possible_types() {
                self.content
                    .entry(obj.id.into())
                    .and_modify(|entry| {
                        entry.score *= score;
                        entry.depth = entry.depth.min(depth);
                        entry.interfaces = true;
                    })
                    .or_insert(SdlOptions {
                        interfaces: true,
                        fields_subset: None,
                        score: self.params.compute_initial_score(ty.name(), depth, score),
                        depth,
                    });
            }
        }
        if let Some(entity) = ty.as_entity() {
            if self.content[&ty.id()].fields_subset.is_none() {
                for field in entity.fields() {
                    self.fields_by_depth[depth as usize].push((field.id, score));
                }
            }
        }
    }

    fn generate_content_for_field(&mut self, field: FieldDefinition<'_>, depth: u8, score: f32) {
        for arg in field.arguments() {
            self.generate_content_for_type(arg.ty().definition(), depth, score);
        }

        let depth = depth + 1;
        if depth > self.params.max_depth || score <= f32::EPSILON {
            return;
        }
        match field.ty().definition_id {
            TypeDefinitionId::Enum(_) | TypeDefinitionId::InputObject(_) => (),
            TypeDefinitionId::Interface(_)
            | TypeDefinitionId::Object(_)
            | TypeDefinitionId::Scalar(_)
            | TypeDefinitionId::Union(_) => {
                self.generate_content_for_type(field.ty().definition(), depth, score);
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
        let mut types = content.into_iter().collect::<Vec<_>>();
        types.sort_unstable_by_key(|(_, op)| op.depth);

        let required_end = types.partition_point(|(_, opt)| opt.depth == 0);
        let (required, optional) = types.split_at_mut(required_end);

        for (id, opt) in required {
            tracing::debug!("{} {}|{}", schema.walk(*id).name(), opt.score, opt.depth);
            buffer.write_type_definition(schema.walk(*id), opt);
        }

        optional.sort_unstable_by(|(_, a), (_, b)| b.score_with_depth().total_cmp(&a.score_with_depth()));
        let mut optional = optional.iter();
        while let Some((id, opt)) = optional.next()
        // .filter(|_| buffer.len() < params.max_size_for_extra_content)
        {
            tracing::debug!(
                "{} {}|{} -> {}",
                schema.walk(id).name(),
                opt.score,
                opt.depth,
                opt.score_with_depth()
            );
            if buffer.len() < params.max_size_for_extra_content {
                buffer.write_type_definition(schema.walk(*id), opt);
            }
        }

        buffer.into_string()
    }
}

impl PartialSdl {
    fn compute_initial_score(&self, name: &str, depth: u8, score: f32) -> f32 {
        let extra = (depth > 0) && self.search_tokens.iter().any(|token| name.contains(token));
        score + (extra as u8) as f32
    }
}

impl SdlOptions {
    fn score_with_depth(&self) -> f32 {
        self.score / (self.depth + 1) as f32
    }
}
