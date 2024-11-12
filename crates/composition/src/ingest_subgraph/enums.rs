use super::*;

pub(super) fn ingest_enum(
    definition_id: DefinitionId,
    enum_type: ast::EnumDefinition<'_>,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &DirectiveMatcher<'_>,
    subgraph: SubgraphId,
) {
    for value in enum_type.values() {
        let value_name = subgraphs.strings.intern(value.value());
        let value_directives = subgraphs.new_directive_site();

        subgraphs.push_enum_value(definition_id, value_name, value_directives);

        directives::ingest_directives(
            value_directives,
            value.directives(),
            subgraphs,
            federation_directives_matcher,
            subgraph,
            |subgraphs| subgraphs.walk(definition_id).name().as_str().to_owned(),
        );
    }
}
