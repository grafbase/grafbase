use crate::diagnostics::CompositeSchemasErrorCode;

use super::*;

/// https://graphql.github.io/composite-schemas-spec/draft/#sec-Query-Root-Type-Inaccessible
pub(crate) fn query_root_type_inaccessible(ctx: &mut ValidateContext<'_>) {
    for subgraph in ctx.subgraphs.iter_subgraph_views() {
        let Some(query_root) = subgraph.query_type else {
            continue;
        };

        let directives = ctx.subgraphs.at(query_root).directives;

        if !directives.inaccessible(ctx.subgraphs) {
            continue;
        }

        let subgraph_name = &ctx.subgraphs[subgraph.name];
        ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
            subgraph_name,
            format_args!("The query root type cannot be inaccessible"),
            CompositeSchemasErrorCode::QueryRootTypeInaccessible,
        );
    }
}
