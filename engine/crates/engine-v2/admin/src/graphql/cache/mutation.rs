use async_graphql::Context;
use async_runtime::make_send_on_wasm;
use engine::registry::CacheTag;

use self::output::CachePurgeAll;

use super::super::super::{error::AdminError, graphql::cache::mutation::input::CachePurgeTypesInput, AdminContext};

mod input {
    #[derive(Debug, async_graphql::InputObject)]
    pub struct PurgeEntityFieldsInput {
        pub name: String,
        pub value: String,
    }

    #[derive(Debug, async_graphql::InputObject)]
    pub struct PurgeEntityInput {
        #[graphql(name = "type")]
        pub cache_type: String,
        pub fields: Vec<PurgeEntityFieldsInput>,
    }

    #[derive(Debug, async_graphql::InputObject)]
    pub struct PurgeListInput {
        #[graphql(name = "type")]
        pub cache_type: String,
    }

    #[derive(Debug, async_graphql::InputObject)]
    pub struct PurgeTypeInput {
        #[graphql(name = "type")]
        pub cache_type: String,
    }

    #[derive(Debug, async_graphql::OneofObject)]
    pub enum CachePurgeTypesInput {
        Type(PurgeTypeInput),
        List(PurgeListInput),
        Entity(PurgeEntityInput),
    }
}

mod output {
    #[derive(Debug, async_graphql::SimpleObject)]
    pub struct CachePurgeTypes {
        pub tags: Vec<String>,
    }

    #[derive(Debug, async_graphql::SimpleObject)]
    pub struct CachePurgeAll {
        pub purged: bool,
    }
}

#[derive(Debug, Default)]
pub struct CachePurgeMutation;

#[async_graphql::Object]
impl CachePurgeMutation {
    pub async fn cache_purge_types(
        &self,
        ctx: &Context<'_>,
        input: CachePurgeTypesInput,
    ) -> Result<output::CachePurgeTypes, AdminError> {
        let ctx = ctx
            .data::<AdminContext>()
            .map_err(|_| AdminError::CachePurgeError("Missing context".to_string()))?;

        let tags: Vec<String> = match input {
            CachePurgeTypesInput::Type(type_purge) => vec![CacheTag::Type {
                type_name: type_purge.cache_type,
            }
            .to_string()],
            CachePurgeTypesInput::List(list_purge) => vec![CacheTag::List {
                type_name: list_purge.cache_type,
            }
            .to_string()],
            CachePurgeTypesInput::Entity(entity_purge) => entity_purge
                .fields
                .into_iter()
                .map(|field| {
                    CacheTag::Field {
                        type_name: entity_purge.cache_type.clone(),
                        field_name: field.name,
                        value: field.value,
                    }
                    .to_string()
                })
                .collect(),
        };

        log::info!(ctx.ray_id, "Purging cache tags: {:?}", tags);

        make_send_on_wasm(ctx.cache.purge_by_tags(tags.clone()))
            .await
            .map_err(|e| AdminError::CachePurgeError(e.to_string()))?;

        log::info!(ctx.ray_id, "Successfully purged tags");

        Ok(output::CachePurgeTypes { tags })
    }

    pub async fn cache_purge_all(&self, ctx: &Context<'_>) -> Result<CachePurgeAll, AdminError> {
        let ctx = ctx
            .data::<AdminContext>()
            .map_err(|_| AdminError::CachePurgeError("Missing context".to_string()))?;

        log::info!(ctx.ray_id, "Purging all cache");

        make_send_on_wasm(ctx.cache.purge_all())
            .await
            .map_err(|e| AdminError::CachePurgeError(e.to_string()))?;

        log::info!(ctx.ray_id, "Successfully purged host");

        Ok(CachePurgeAll { purged: true })
    }
}
