use std::{any::TypeId, borrow::Cow, collections::HashSet};

use crate::federation::{DockerSubgraph, MockSubgraph};
use cynic_introspection::{IntrospectionQuery, SpecificationVersion};
use futures::{future::BoxFuture, stream::FuturesUnordered, StreamExt};
use graphql_mocks::MockGraphQlServer;
use url::Url;

pub(super) struct Subgraphs(Vec<Subgraph>);

impl Subgraphs {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Subgraph> + '_ {
        self.0.iter()
    }

    pub fn get_mock_by_type<S: graphql_mocks::Subgraph>(&self) -> Option<&MockSubgraph> {
        let target = std::any::TypeId::of::<S>();
        self.0.iter().find_map(|subgraph| match subgraph {
            Subgraph::Mock { type_id, server } if *type_id == target => Some(server),
            _ => None,
        })
    }

    pub async fn load(
        mock_subgraphs: Vec<(TypeId, String, BoxFuture<'static, MockGraphQlServer>)>,
        docker_subgraphs: HashSet<DockerSubgraph>,
    ) -> Self {
        let mock_subgraphs_fut = mock_subgraphs
            .into_iter()
            .map(|(type_id, name, server)| async move {
                Subgraph::Mock {
                    type_id,
                    server: MockSubgraph {
                        name,
                        server: server.await,
                    },
                }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>();

        let docker_subgraphs_fut = docker_subgraphs
            .into_iter()
            .map(|subgraph| async move {
                let request = IntrospectionQuery::with_capabilities(SpecificationVersion::October2021.capabilities());
                #[derive(serde::Deserialize)]
                struct Response {
                    data: IntrospectionQuery,
                }
                let sdl = reqwest::Client::new()
                    .post(subgraph.url())
                    .json(&request)
                    .send()
                    .await
                    .unwrap()
                    .error_for_status()
                    .unwrap()
                    .json::<Response>()
                    .await
                    .unwrap()
                    .data
                    .into_schema()
                    .unwrap()
                    .to_sdl();
                Subgraph::Docker { sdl, subgraph }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>();

        let (mut subgraphs, docker_subgraphs) = futures::join!(mock_subgraphs_fut, docker_subgraphs_fut);
        subgraphs.extend(docker_subgraphs);

        // Ensures consistency of composition and thus introspection tests.
        subgraphs.sort_unstable_by(|a, b| a.name().cmp(b.name()));

        Self(subgraphs)
    }
}

pub(super) enum Subgraph {
    Mock { type_id: TypeId, server: MockSubgraph },
    Docker { subgraph: DockerSubgraph, sdl: String },
}

impl Subgraph {
    pub fn name(&self) -> &str {
        match self {
            Subgraph::Mock { server, .. } => &server.name,
            Subgraph::Docker { subgraph, .. } => subgraph.name(),
        }
    }

    pub fn sdl(&self) -> Cow<'_, str> {
        match self {
            Subgraph::Mock { server, .. } => server.sdl().into(),
            Subgraph::Docker { sdl, .. } => sdl.into(),
        }
    }

    pub fn url(&self) -> Url {
        match self {
            Subgraph::Mock { server, .. } => server.url(),
            Subgraph::Docker { subgraph, .. } => subgraph.url(),
        }
    }

    pub fn websocket_url(&self) -> Url {
        match self {
            Subgraph::Mock { server, .. } => server.websocket_url(),
            Subgraph::Docker { subgraph, .. } => subgraph.url(),
        }
    }
}
