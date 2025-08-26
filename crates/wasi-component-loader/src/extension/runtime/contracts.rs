use engine_schema::{
    DirectiveSiteId, EntityDefinitionId, EnumDefinitionId, EnumValueId, FieldDefinitionId, Inaccessible,
    InputObjectDefinitionId, InputValueDefinitionId, InputValueParentDefinitionId, InterfaceDefinitionId,
    MutableSchema, ObjectDefinitionId, ScalarDefinitionId, Schema, TypeDefinition, TypeSystemDirective,
    UnionDefinitionId,
};
use extension_catalog::ExtensionId;
use id_newtypes::IdToOne;
use itertools::Itertools as _;
use rapidhash::fast::RapidHashMap;
use runtime::extension::ContractsExtension;

use crate::{
    cbor,
    extension::{EngineWasmExtensions, api::wit},
    wasmsafe,
};

impl ContractsExtension for EngineWasmExtensions {
    async fn construct(&self, key: String, schema: Schema) -> Option<Schema> {
        let mut instance = match self.contracts().await {
            Ok(Some(instance)) => instance,
            Ok(None) => {
                tracing::error!("No contract extension defined");
                return None;
            }
            Err(err) => {
                tracing::error!("Failed to get instance for contract extension: {err}");
                return None;
            }
        };

        let (directives, sites_by_directive) = SiteIngester::ingest(instance.store().data().extension_id(), &schema);
        let n_directives = directives.len();
        let subgraphs = schema
            .subgraphs()
            .filter_map(|sg| sg.as_graphql())
            .map(|gql| wit::GraphqlSubgraphParam {
                name: gql.name(),
                url: gql.url().as_str(),
            })
            .collect();

        let mut contract = match wasmsafe!(instance.construct(&key, &directives, subgraphs).await) {
            Ok(contract) => contract,
            Err(err) => {
                tracing::error!("Failed to construct contract for key {key}: {err}");
                return None;
            }
        };

        if contract.accessible.len() < n_directives {
            contract.accessible.resize(n_directives, -1);
        } else {
            // Just in case we truncate the rest.
            contract.accessible.truncate(n_directives);
        }

        let mut output = contract
            .accessible
            .into_iter()
            .enumerate()
            .map(|(ix, accessible)| DirectiveResult {
                priority: priority(accessible),
                accessible,
                index: ix as u32,
            })
            .collect::<Vec<_>>();
        output.sort_unstable_by(|a, b| a.priority.cmp(&b.priority));

        tracing::debug!(
            "Contract(hide_unreachable_types: {}, accessible_by_default: {})\n{}",
            contract.hide_unreachable_types,
            contract.accessible_by_default,
            output
                .iter()
                .format_with("\n", |result, f| {
                    let directive = &directives[result.index as usize];
                    f(&format_args!(
                        "{}({}): {} -> {}",
                        directive.name,
                        serde_json::to_string(&cbor::from_slice::<serde_json::Value>(&directive.arguments).unwrap())
                            .unwrap(),
                        result.accessible,
                        sites_by_directive[result.index as usize]
                            .iter()
                            .format_with(",", |id, f| f(&format_args!("{}", schema.walk(*id))))
                    ))
                })
                .to_string() // this panics otherwise if opentelemetry is enabled
        );

        let mut ingester = InaccessibilityIngester::new(schema, contract.accessible_by_default);

        for DirectiveResult { accessible, index, .. } in output {
            for site_id in &sites_by_directive[index as usize] {
                ingester.ingest(*site_id, accessible);
            }
        }
        let mut schema = ingester.into_mutable_schema();

        for gql in contract.subgraphs {
            let url = match gql.url.parse() {
                Ok(url) => url,
                Err(err) => {
                    tracing::error!("Invalid URL for subgraph {}: {err}", gql.name);
                    return None;
                }
            };
            schema.update_graphql_endpoint(&gql.name, url);
        }

        let schema = schema.finalize(contract.hide_unreachable_types);

        Some(schema)
    }
}

struct DirectiveResult {
    priority: u8,
    accessible: i8,
    index: u32,
}

pub fn priority(included: i8) -> u8 {
    // [0, 127] => accessible
    // [-128, -1] => inaccessible
    //
    // As accessible is shifted down by one to use the complete i8 range. So we add 1
    // back for accessible case (positive) to have the [1, 128] range for priorities.
    let accessible = included >= 0;
    included.unsigned_abs() + accessible as u8
}

struct SiteIngester<'a> {
    sites: RapidHashMap<wit::Directive<'a>, Vec<DirectiveSiteId>>,
    id: ExtensionId,
}

impl<'a> SiteIngester<'a> {
    fn ingest(id: ExtensionId, schema: &'a Schema) -> (Vec<wit::Directive<'a>>, Vec<Vec<DirectiveSiteId>>) {
        let mut ingester = Self {
            sites: RapidHashMap::default(),
            id,
        };
        for ty in schema.type_definitions() {
            match ty {
                TypeDefinition::Enum(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                    for value in def.values() {
                        ingester.ingest_site(value.id.into(), value.directives());
                    }
                }
                TypeDefinition::InputObject(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                    for field in def.input_fields() {
                        ingester.ingest_site(field.id.into(), field.directives());
                    }
                }
                TypeDefinition::Interface(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                    for field in def.fields() {
                        ingester.ingest_site(field.id.into(), field.directives());
                        for arg in field.arguments() {
                            ingester.ingest_site(arg.id.into(), arg.directives());
                        }
                    }
                }
                TypeDefinition::Object(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                    for field in def.fields() {
                        ingester.ingest_site(field.id.into(), field.directives());
                        for arg in field.arguments() {
                            ingester.ingest_site(arg.id.into(), arg.directives());
                        }
                    }
                }
                TypeDefinition::Scalar(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                }
                TypeDefinition::Union(def) => {
                    ingester.ingest_site(def.id.into(), def.directives());
                }
            }
        }

        ingester.sites.into_iter().unzip()
    }

    fn ingest_site(&mut self, site_id: DirectiveSiteId, directives: impl Iterator<Item = TypeSystemDirective<'a>>) {
        directives
            .filter_map(|dir| dir.as_extension())
            .filter(|dir| dir.extension_id == self.id)
            .map(|dir| {
                let args = cbor::to_vec(dir.static_arguments()).expect("Valid schema");
                wit::Directive {
                    name: dir.name(),
                    arguments: args,
                }
            })
            .for_each(|arg| self.sites.entry(arg).or_default().push(site_id));
    }
}

struct InaccessibilityIngester {
    object_definitions: IdToOne<ObjectDefinitionId, i8>,
    interface_definitions: IdToOne<InterfaceDefinitionId, i8>,
    field_definitions: IdToOne<FieldDefinitionId, i8>,
    enum_definitions: IdToOne<EnumDefinitionId, i8>,
    enum_values: IdToOne<EnumValueId, i8>,
    input_object_definitions: IdToOne<InputObjectDefinitionId, i8>,
    input_value_definitions: IdToOne<InputValueDefinitionId, i8>,
    scalar_definitions: IdToOne<ScalarDefinitionId, i8>,
    union_definitions: IdToOne<UnionDefinitionId, i8>,
    schema: MutableSchema,
    inaccessible: Inaccessible,
}

impl InaccessibilityIngester {
    pub fn new(schema: Schema, accessible_by_default: bool) -> Self {
        let default = accessible_by_default as i8 - 1;
        let schema = schema.into_mutable();
        let inaccessible = schema.new_inaccessible();
        Self {
            object_definitions: IdToOne::init(default, inaccessible.object_definitions.len()),
            interface_definitions: IdToOne::init(default, inaccessible.interface_definitions.len()),
            field_definitions: IdToOne::init(default, inaccessible.field_definitions.len()),
            enum_definitions: IdToOne::init(default, inaccessible.enum_definitions.len()),
            enum_values: IdToOne::init(default, inaccessible.enum_values.len()),
            input_object_definitions: IdToOne::init(default, inaccessible.input_object_definitions.len()),
            input_value_definitions: IdToOne::init(default, inaccessible.input_value_definitions.len()),
            scalar_definitions: IdToOne::init(default, inaccessible.scalar_definitions.len()),
            union_definitions: IdToOne::init(default, inaccessible.union_definitions.len()),
            schema,
            inaccessible,
        }
    }

    pub fn ingest(&mut self, site_id: DirectiveSiteId, accessible: i8) {
        match site_id {
            DirectiveSiteId::Enum(id) => {
                self.enum_definitions[id] = accessible;
            }
            DirectiveSiteId::EnumValue(id) => {
                self.enum_values[id] = accessible;
            }
            DirectiveSiteId::Field(id) => {
                self.field_definitions[id] = accessible;
            }
            DirectiveSiteId::InputObject(id) => {
                self.input_object_definitions[id] = accessible;
            }
            DirectiveSiteId::InputValue(id) => {
                self.input_value_definitions[id] = accessible;
            }
            DirectiveSiteId::Interface(id) => {
                self.interface_definitions[id] = accessible;
            }
            DirectiveSiteId::Object(id) => {
                self.object_definitions[id] = accessible;
            }
            DirectiveSiteId::Scalar(id) => {
                self.scalar_definitions[id] = accessible;
            }
            DirectiveSiteId::Union(id) => {
                self.union_definitions[id] = accessible;
            }
        }
    }

    pub fn into_mutable_schema(mut self) -> MutableSchema {
        let mut schema = self.schema;
        let mut inaccessible = self.inaccessible;

        // == First propagate downwards. ==

        for (id, accessible) in self.input_object_definitions.iter() {
            for field_id in schema.walk(id).input_field_ids {
                replace_if_higher_priority(&mut self.input_value_definitions[field_id], *accessible);
            }
        }

        for (id, accessible) in self.object_definitions.iter() {
            for field_id in schema.walk(id).field_ids {
                replace_if_higher_priority(&mut self.field_definitions[field_id], *accessible);
            }
        }

        for (id, accessible) in self.interface_definitions.iter() {
            for field_id in schema.walk(id).field_ids {
                replace_if_higher_priority(&mut self.field_definitions[field_id], *accessible);
            }
        }

        for (id, accessible) in self.field_definitions.iter() {
            for arg_id in schema.walk(id).argument_ids {
                replace_if_higher_priority(&mut self.input_value_definitions[arg_id], *accessible);
            }
        }

        for (id, accessible) in self.enum_definitions.iter() {
            for value_id in schema.walk(id).value_ids {
                replace_if_higher_priority(&mut self.enum_values[value_id], *accessible);
            }
        }

        // == Then propagate upwards if accessible. ==

        for (id, accessible) in self.enum_values {
            inaccessible.enum_values.set(id, accessible < 0);
            if accessible >= 0 {
                let id = schema.walk(id).parent_enum_id;
                replace_if_higher_priority(&mut self.enum_definitions[id], accessible);
            }
        }

        for (id, accessible) in self.input_value_definitions {
            inaccessible.input_value_definitions.set(id, accessible < 0);
            if accessible >= 0 {
                match schema.walk(id).parent_id {
                    InputValueParentDefinitionId::Field(id) => {
                        replace_if_higher_priority(&mut self.field_definitions[id], accessible);
                        match schema.walk(id).parent_entity_id {
                            EntityDefinitionId::Interface(id) => {
                                replace_if_higher_priority(&mut self.interface_definitions[id], accessible);
                            }
                            EntityDefinitionId::Object(id) => {
                                replace_if_higher_priority(&mut self.object_definitions[id], accessible);
                            }
                        }
                    }
                    InputValueParentDefinitionId::InputObject(id) => {
                        replace_if_higher_priority(&mut self.input_object_definitions[id], accessible);
                    }
                }
            }
        }

        for (id, accessible) in self.field_definitions {
            inaccessible.field_definitions.set(id, accessible < 0);
            if accessible >= 0 {
                match schema.walk(id).parent_entity_id {
                    EntityDefinitionId::Interface(id) => {
                        replace_if_higher_priority(&mut self.interface_definitions[id], accessible);
                    }
                    EntityDefinitionId::Object(id) => {
                        replace_if_higher_priority(&mut self.object_definitions[id], accessible);
                    }
                }
            }
        }

        // == Finally, set the inaccessible flags for everything else ==

        for (id, accessible) in self.enum_definitions {
            inaccessible.enum_definitions.set(id, accessible < 0);
        }

        for (id, accessible) in self.input_object_definitions {
            inaccessible.input_object_definitions.set(id, accessible < 0);
        }

        for (id, accessible) in self.object_definitions {
            inaccessible.object_definitions.set(id, accessible < 0);
        }

        for (id, accessible) in self.interface_definitions {
            inaccessible.interface_definitions.set(id, accessible < 0);
        }

        for (id, accessible) in self.union_definitions {
            inaccessible.union_definitions.set(id, accessible < 0);
        }

        for (id, accessible) in self.scalar_definitions {
            inaccessible.scalar_definitions.set(id, accessible < 0);
        }

        schema.update_inaccessible(inaccessible);

        schema
    }
}

fn replace_if_higher_priority(current: &mut i8, new: i8) {
    let replace = priority(*current) < priority(new);
    let keep_current_mask = replace as i8 - 1; // 0xFF if false, 0x00 if true
    *current = (!keep_current_mask & new) | (keep_current_mask & *current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_test() {
        assert_eq!(priority(0), 1);
        assert_eq!(priority(1), 2);
        assert_eq!(priority(127), 128);
        assert_eq!(priority(-1), 1);
        assert_eq!(priority(-2), 2);
        assert_eq!(priority(-128), 128);
    }

    #[test]
    fn replace_if_higher_priority_test() {
        let mut current = 1;
        replace_if_higher_priority(&mut current, 2);
        assert_eq!(current, 2);

        current = 2;
        replace_if_higher_priority(&mut current, 1);
        assert_eq!(current, 2);

        current = -1;
        replace_if_higher_priority(&mut current, -2);
        assert_eq!(current, -2);

        current = -2;
        replace_if_higher_priority(&mut current, -1);
        assert_eq!(current, -2);

        current = -5;
        replace_if_higher_priority(&mut current, 10);
        assert_eq!(current, 10);

        current = 5;
        replace_if_higher_priority(&mut current, -10);
        assert_eq!(current, -10);

        current = 0;
        replace_if_higher_priority(&mut current, 1);
        assert_eq!(current, 1);

        current = -1;
        replace_if_higher_priority(&mut current, 1);
        assert_eq!(current, 1);

        current = 0;
        replace_if_higher_priority(&mut current, -2);
        assert_eq!(current, -2);

        current = -1;
        replace_if_higher_priority(&mut current, -2);
        assert_eq!(current, -2);
    }
}
