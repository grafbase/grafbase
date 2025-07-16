use engine_error::{ErrorCode, ErrorResponse, GraphqlError};
use engine_schema::{DirectiveSiteId, Schema, TypeDefinition, TypeSystemDirective};
use extension_catalog::ExtensionId;
use rapidhash::RapidHashMap;
use runtime::extension::ContractsExtension;

use crate::{
    Error, SharedContext, cbor,
    extension::{EngineWasmExtensions, api::wit},
};

impl ContractsExtension<SharedContext> for EngineWasmExtensions {
    async fn construct(&self, context: &SharedContext, key: String, schema: Schema) -> Result<Schema, ErrorResponse> {
        let Some(mut instance) = self.contracts().await? else {
            tracing::error!("Missing contracts extensions, cannot handle contract key: {key}");
            return Err(ErrorResponse::new(http::StatusCode::INTERNAL_SERVER_ERROR)
                .with_error(GraphqlError::internal_server_error()));
        };

        let (directives, sites_by_directive) = SiteIngester::ingest(instance.store().data().extension_id(), &schema);

        let contract = instance
            .construct(
                context.clone(),
                directives,
                schema
                    .subgraphs()
                    .filter_map(|sg| sg.as_graphql_endpoint())
                    .map(|gql| wit::GraphqlSubgraphParam {
                        name: gql.subgraph_name(),
                        url: gql.url().as_str(),
                    })
                    .collect(),
            )
            .await
            .map_err(|err| match err {
                Error::Internal(err) => {
                    tracing::error!("Wasm error: {err}");
                    GraphqlError::internal_extension_error()
                }
                Error::Guest(err) => err.into_graphql_error(ErrorCode::ExtensionError),
            })?
            .map_err(|err| {
                tracing::error!("Could not build contract: {err}");
                GraphqlError::internal_extension_error()
            })?;

        let mut schema = schema.into_mutable();
        if !contract.accessible_by_default {
            schema.mark_all_as_inaccessible();
        }

        let mut output = contract
            .accessible
            .into_iter()
            .enumerate()
            .map(|(ix, output)| {
                let priority = output.unsigned_abs();
                let accessible = output > 0;
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
            let url = gql.url.parse().map_err(|err| {
                tracing::error!("Invalid URL for subgraph {}: {err}", gql.name);
                GraphqlError::internal_extension_error()
            })?;
            schema.update_graphql_endpoint(&gql.name, url);
        }

        let schema = schema.finalize();

        Ok(schema)
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
