use builder::{
    interner::{Interner, ProxyKeyInterner},
    subgraphs::SubgraphsBuilder,
};
use extension_catalog::Extension;
use gateway_config::Config;
use url::Url;

use crate::*;

use super::{extension::ExtensionsContext, sdl::Sdl};

#[derive(id_derives::IndexedFields)]
pub(crate) struct BuildContext<'a> {
    pub sdl: &'a Sdl<'a>,
    pub extensions: &'a ExtensionsContext<'a>,
    pub config: &'a Config,
    pub interners: Interners,
    pub subgraphs: SubgraphsBuilder<'a>,
}

#[derive(Default)]
pub(crate) struct Interners {
    pub strings: Interner<String, StringId>,
    pub regexps: ProxyKeyInterner<Regex, RegexId>,
    pub urls: Interner<Url, UrlId>,
}

id_newtypes::forward! {
    impl Index<StringId, Output = String> for BuildContext<'a>.interners.strings,
    impl Index<RegexId, Output = Regex> for BuildContext<'a>.interners.regexps,
    impl Index<UrlId, Output = Url> for BuildContext<'a>.interners.urls,
    impl Index<VirtualSubgraphId, Output = VirtualSubgraphRecord> for BuildContext<'a>.subgraphs,
    impl Index<GraphqlEndpointId, Output = GraphqlEndpointRecord> for BuildContext<'a>.subgraphs,
    impl Index<ExtensionId, Output = Extension> for BuildContext<'a>.extensions.catalog,
}

impl<'a> BuildContext<'a> {
    pub fn new(
        sdl: &'a Sdl<'a>,
        extensions: &'a ExtensionsContext<'a>,
        config: &'a Config,
    ) -> Result<Self, BuildError> {
        let mut interners = Interners::default();
        let subgraphs = SubgraphsBuilder::new(sdl, config, &mut interners)?;
        Ok(Self {
            sdl,
            extensions,
            config,
            interners,
            subgraphs,
        })
    }
    pub(crate) fn ingest_str(&mut self, s: impl AsRef<str>) -> StringId {
        self.interners.strings.get_or_new(s.as_ref())
    }
}
