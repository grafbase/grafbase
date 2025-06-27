use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    RequiresScopesDirectiveRecord, TypeSystemDirectiveId,
    builder::{Error, graph::directives::DirectivesIngester, sdl},
};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn create_requires_scopes_directive(
        &mut self,
        _def: sdl::SdlDefinition<'sdl>,
        directive: sdl::Directive<'sdl>,
    ) -> Result<TypeSystemDirectiveId, Error> {
        let dir = directive.deserialize::<sdl::RequiresScopesDirective>().map_err(|err| {
            (
                format!("Invalid @requiresScopes directive: {err}"),
                directive.arguments_span(),
            )
        })?;
        let scope = RequiresScopesDirectiveRecord::new(
            dir.scopes
                .into_iter()
                .map(|scopes| scopes.into_iter().map(|scope| self.ingest_str(scope)).collect())
                .collect(),
        );
        let id = self.required_scopes.get_or_insert(scope);
        Ok(TypeSystemDirectiveId::RequiresScopes(id))
    }
}
