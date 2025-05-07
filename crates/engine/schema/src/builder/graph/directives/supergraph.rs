use crate::builder::{Error, sdl};

use super::DirectivesIngester;

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn ingest_federation_aware_directives(
        &mut self,
        def: sdl::SdlDefinition<'sdl>,
        directives: &[sdl::Directive<'sdl>],
    ) -> Result<(), Error> {
        for &directive in directives {
            match directive.name() {
                name if name.starts_with("composite__") => self
                    .ingest_composite_directive_after_federation(def, directive)
                    .map_err(|err| err.with_span_if_absent(directive.arguments_span()))?,
                _ => {}
            }
        }
        Ok(())
    }
}
