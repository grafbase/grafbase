use super::*;

impl Context<'_> {
    pub(crate) fn ingest_extra_directive_arguments(
        &mut self,
        arguments: ast::Iter<'_, ast::Argument<'_>>,
    ) -> Vec<(subgraphs::StringId, subgraphs::Value)> {
        arguments
            .map(|argument| {
                (
                    self.subgraphs.strings.intern(argument.name()),
                    ast_value_to_subgraph_value(argument.value(), self.subgraphs),
                )
            })
            .collect()
    }
}
