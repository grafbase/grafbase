use crate::{
    dev::{admin::Header, composer::Subgraph},
    Error,
};
use async_graphql_parser::types::ServiceDocument;
use tokio::sync::oneshot;
use url::Url;

pub(crate) type ResponseSender<T> = oneshot::Sender<Result<T, Error>>;

pub(crate) enum RecomposeDescription {
    Removed(String),
}

pub(crate) enum ComposeMessage {
    Introspect(IntrospectSchema),
    Compose(ComposeSchema),
    RemoveSubgraph(RemoveSubgraph),
    Recompose(RecomposeDescription),
    InitializeRefresh,
}

impl From<IntrospectSchema> for ComposeMessage {
    fn from(value: IntrospectSchema) -> Self {
        Self::Introspect(value)
    }
}

impl From<ComposeSchema> for ComposeMessage {
    fn from(value: ComposeSchema) -> Self {
        Self::Compose(value)
    }
}

impl From<RemoveSubgraph> for ComposeMessage {
    fn from(value: RemoveSubgraph) -> Self {
        Self::RemoveSubgraph(value)
    }
}

pub(crate) struct ComposeSchema {
    name: String,
    subgraph: Subgraph,
    responder: ResponseSender<()>,
}

impl ComposeSchema {
    pub(crate) fn new(name: String, subgraph: Subgraph, responder: ResponseSender<()>) -> Self {
        Self {
            name,
            subgraph,
            responder,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn subgraph(&self) -> &Subgraph {
        &self.subgraph
    }

    pub(crate) fn parts(&self) -> (&str, &Subgraph) {
        (self.name(), self.subgraph())
    }

    pub(crate) fn into_parts(self) -> (String, Subgraph, ResponseSender<()>) {
        (self.name, self.subgraph, self.responder)
    }
}

pub(crate) struct RemoveSubgraph {
    name: String,
}

impl RemoveSubgraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

pub(crate) struct IntrospectSchema {
    name: String,
    url: Url,
    responder: ResponseSender<ServiceDocument>,
    headers: Vec<Header>,
}

impl IntrospectSchema {
    pub(crate) fn new(
        name: impl Into<String>,
        url: Url,
        responder: ResponseSender<ServiceDocument>,
        headers: Vec<Header>,
    ) -> Self {
        Self {
            name: name.into(),
            url,
            responder,
            headers,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn into_parts(self) -> (String, Url, Vec<Header>, ResponseSender<ServiceDocument>) {
        (self.name, self.url, self.headers, self.responder)
    }
}
