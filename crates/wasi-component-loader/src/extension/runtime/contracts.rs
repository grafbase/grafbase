use engine_schema::{DirectiveSiteId, Schema, TypeDefinition, TypeSystemDirective};
use extension_catalog::ExtensionId;
use rapidhash::RapidHashMap;
use runtime::extension::ContractsExtension;

use crate::{
    WasmContext, cbor,
    extension::{EngineWasmExtensions, api::wit},
    wasmsafe,
};

impl ContractsExtension<WasmContext> for EngineWasmExtensions {
    async fn construct(&self, context: &WasmContext, key: String, schema: Schema) -> Option<Schema> {
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
            .filter_map(|sg| sg.as_graphql_endpoint())
            .map(|gql| wit::GraphqlSubgraphParam {
                name: gql.subgraph_name(),
                url: gql.url().as_str(),
            })
            .collect();

        let mut contract = match wasmsafe!(instance.construct(context, &key, directives, subgraphs).await) {
            Ok(contract) => contract,
            Err(err) => {
                tracing::error!("Failed to construct contract for key {key}: {err}");
                return None;
            }
        };

        let mut schema = schema.into_mutable();
        if !contract.accessible_by_default {
            schema.mark_all_as_inaccessible();
        }
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
            .map(|(ix, output)| {
                // [0, 127] => accessible
                // [-128, -1] => inaccessible
                //
                // As accessible is shifted down by one to use the complete i8 range. So we add 1
                // back for accessible case (positive) to have the [1, 128] range for priorities.
                let accessible = output >= 0;
                let priority = output.unsigned_abs() + accessible as u8;
                DirectiveResult {
                    priority,
                    accessible,
                    index: ix as u32,
                }
            })
            .collect::<Vec<_>>();
        output.sort_unstable_by(|a, b| a.priority.cmp(&b.priority));

        for DirectiveResult { accessible, index, .. } in output {
            for site_id in &sites_by_directive[index as usize] {
                schema.mark_as_accessible(*site_id, accessible);
            }
        }

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
    accessible: bool,
    index: u32,
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
