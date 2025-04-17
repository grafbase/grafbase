use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    DeprecatedDirectiveRecord, TypeSystemDirectiveId,
    builder::{Error, graph::directives::DirectivesIngester, sdl},
};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn create_deprecated_directive(
        &mut self,
        _def: sdl::SdlDefinition<'sdl>,
        directive: sdl::Directive<'sdl>,
    ) -> Result<TypeSystemDirectiveId, Error> {
        let dir = directive.deserialize::<sdl::DeprecatedDirective>().map_err(|err| {
            (
                format!("Invalid @deprecated directive: {}", err),
                directive.arguments_span(),
            )
        })?;
        let reason_id = dir.reason.map(|reason| self.ingest_str(reason));
        Ok(TypeSystemDirectiveId::Deprecated(DeprecatedDirectiveRecord {
            reason_id,
        }))
    }
}
