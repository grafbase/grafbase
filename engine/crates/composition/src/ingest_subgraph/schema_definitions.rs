use super::*;

pub(super) fn ingest_schema_definition(document: &ast::ServiceDocument) -> RootTypeMatcher<'_> {
    let schema_definitions = document.definitions.iter().filter_map(|definition| match &definition {
        ast::TypeSystemDefinition::Schema(schema_definition) => Some(schema_definition),
        _ => None,
    });

    let mut matcher = RootTypeMatcher::default();

    for schema_definition in schema_definitions {
        let node = &schema_definition.node;

        matcher.query = matcher.query.or(node.query.as_ref().map(|query| query.node.as_str()));
        matcher.mutation = matcher
            .mutation
            .or(node.mutation.as_ref().map(|query| query.node.as_str()));
        matcher.subscription = matcher
            .subscription
            .or(node.subscription.as_ref().map(|query| query.node.as_str()));
    }

    matcher
}

#[derive(Default)]
pub(super) struct RootTypeMatcher<'a> {
    query: Option<&'a str>,
    mutation: Option<&'a str>,
    subscription: Option<&'a str>,
}

impl RootTypeMatcher<'_> {
    pub(crate) fn is_query(&self, name: &str) -> bool {
        matches!(self.match_name(name), RootTypeMatch::Query)
    }

    pub(crate) fn match_name(&self, name: &str) -> RootTypeMatch {
        for (name_from_definition, default_name, match_case) in [
            (self.query, "Query", RootTypeMatch::Query),
            (self.mutation, "Mutation", RootTypeMatch::Mutation),
            (self.subscription, "Subscription", RootTypeMatch::Subscription),
        ] {
            match name_from_definition {
                Some(root_name) if root_name == name => return match_case,
                None if name == default_name => return match_case,

                Some(_) if name == default_name => return RootTypeMatch::NotRootButHasDefaultRootName,
                Some(_) | None => continue,
            }
        }

        RootTypeMatch::NotRoot
    }
}

pub(crate) enum RootTypeMatch {
    Query,
    Mutation,
    Subscription,
    NotRootButHasDefaultRootName,
    NotRoot,
}
