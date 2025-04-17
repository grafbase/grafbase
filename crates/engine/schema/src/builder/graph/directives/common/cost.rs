use cynic_parser_deser::ConstDeserializer as _;

use crate::{
    CostDirectiveRecord, TypeSystemDirectiveId,
    builder::{Error, graph::directives::DirectivesIngester, sdl},
};

impl<'sdl> DirectivesIngester<'_, 'sdl> {
    pub fn create_cost_directive(
        &mut self,
        _def: sdl::SdlDefinition<'sdl>,
        directive: sdl::Directive<'sdl>,
    ) -> Result<TypeSystemDirectiveId, Error> {
        let dir = directive
            .deserialize::<sdl::CostDirective>()
            .map_err(|err| (format!("Invalid @cost directive: {}", err), directive.arguments_span()))?;
        self.graph
            .cost_directives
            .push(CostDirectiveRecord { weight: dir.weight });
        Ok(TypeSystemDirectiveId::Cost(
            (self.graph.cost_directives.len() - 1).into(),
        ))
    }
}
