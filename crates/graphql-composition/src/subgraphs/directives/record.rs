use super::*;

pub(crate) type ExtraDirective<'a> = View<'a, DirectiveId, ExtraDirectiveRecord>;

/// Directives that aren't part of the built-in directives, that is to say not from the GraphQL spec, link spec or federation spec.
pub(crate) struct ExtraDirectiveRecord {
    pub(crate) directive_site_id: DirectiveSiteId,
    pub(crate) name: StringId,
    pub(crate) arguments: Arguments,
    pub(crate) provenance: DirectiveProvenance,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DirectiveProvenance {
    Linked {
        /// The `@link`ed schema definition it comes from.
        linked_schema_id: LinkedSchemaId,
        /// Has the directive been composed with `@composeDirective`?
        is_composed_directive: bool,
    },
    ComposedDirective,
}
