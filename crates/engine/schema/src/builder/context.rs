use builder::{
    extension::SchemaExtension,
    interner::{Interner, ProxyKeyInterner},
    subgraphs::SubgraphsContext,
};
use extension_catalog::ExtensionCatalog;
use url::Url;

use crate::*;

#[derive(id_derives::IndexedFields)]
pub(crate) struct Context<'a> {
    // -- read-only
    pub config: &'a gateway_config::Config,
    pub federated_graph: &'a federated_graph::FederatedGraph,
    pub extension_catalog: &'a ExtensionCatalog,
    // -- Immediately initialized
    #[indexed_by(federated_graph::ExtensionId)]
    pub extensions: Vec<SchemaExtension>,
    pub subgraphs: SubgraphsContext,
    // --
    pub strings: Interner<String, StringId>,
    pub regexps: ProxyKeyInterner<Regex, RegexId>,
    pub urls: Interner<Url, UrlId>,
    pub header_rules: Vec<HeaderRuleRecord>,
}

impl std::ops::Index<StringId> for Context<'_> {
    type Output = String;
    fn index(&self, id: StringId) -> &String {
        self.strings.get_by_id(id).unwrap()
    }
}

impl std::ops::Index<RegexId> for Context<'_> {
    type Output = Regex;
    fn index(&self, index: RegexId) -> &Regex {
        &self.regexps[index]
    }
}

impl std::ops::Index<UrlId> for Context<'_> {
    type Output = Url;
    fn index(&self, id: UrlId) -> &Url {
        self.urls.get_by_id(id).unwrap()
    }
}

impl<'a> Context<'a> {
    pub(crate) async fn new(
        config: &'a gateway_config::Config,
        extension_catalog: &'a ExtensionCatalog,
        federated_graph: &'a federated_graph::FederatedGraph,
    ) -> Result<Self, BuildError> {
        let mut ctx = Self {
            config,
            extension_catalog,
            federated_graph,
            strings: Interner::with_capacity(federated_graph.strings.len()),
            regexps: Default::default(),
            urls: Interner::with_capacity(federated_graph.subgraphs.len()),
            header_rules: Vec::new(),
            subgraphs: Default::default(),
            extensions: Vec::new(),
        };
        ctx.load_subgraphs()?;
        ctx.load_extension_links().await?;
        Ok(ctx)
    }

    pub(crate) fn get_or_insert_str(&mut self, id: federated_graph::StringId) -> StringId {
        self.strings.get_or_new(&self.federated_graph[id])
    }

    pub(crate) fn ingest_header_rules(&mut self, rules: &[gateway_config::HeaderRule]) -> IdRange<HeaderRuleId> {
        use gateway_config::*;
        let start = self.header_rules.len();
        self.header_rules.extend(rules.iter().map(|rule| -> HeaderRuleRecord {
            match rule {
                HeaderRule::Forward(rule) => {
                    let name_id = match &rule.name {
                        NameOrPattern::Pattern(regex) => {
                            NameOrPatternId::Pattern(self.regexps.get_or_insert(regex.clone()))
                        }
                        NameOrPattern::Name(name) => NameOrPatternId::Name(self.strings.get_or_new(name.as_ref())),
                    };

                    let default_id = rule.default.as_ref().map(|s| self.strings.get_or_new(s.as_ref()));
                    let rename_id = rule.rename.as_ref().map(|s| self.strings.get_or_new(s.as_ref()));

                    HeaderRuleRecord::Forward(ForwardHeaderRuleRecord {
                        name_id,
                        default_id,
                        rename_id,
                    })
                }
                HeaderRule::Insert(rule) => {
                    let name_id = self.strings.get_or_new(rule.name.as_ref());
                    let value_id = self.strings.get_or_new(rule.value.as_ref());

                    HeaderRuleRecord::Insert(InsertHeaderRuleRecord { name_id, value_id })
                }
                HeaderRule::Remove(rule) => {
                    let name_id = match &rule.name {
                        NameOrPattern::Pattern(regex) => {
                            NameOrPatternId::Pattern(self.regexps.get_or_insert(regex.clone()))
                        }
                        NameOrPattern::Name(name) => NameOrPatternId::Name(self.strings.get_or_new(name.as_ref())),
                    };

                    HeaderRuleRecord::Remove(RemoveHeaderRuleRecord { name_id })
                }
                HeaderRule::RenameDuplicate(rule) => {
                    HeaderRuleRecord::RenameDuplicate(RenameDuplicateHeaderRuleRecord {
                        name_id: self.strings.get_or_new(rule.name.as_ref()),
                        default_id: rule
                            .default
                            .as_ref()
                            .map(|default| self.strings.get_or_new(default.as_ref())),
                        rename_id: self.strings.get_or_new(rule.rename.as_ref()),
                    })
                }
            }
        }));
        (start..self.header_rules.len()).into()
    }
}
