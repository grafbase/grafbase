use std::{any::TypeId, borrow::Cow, collections::HashSet, sync::Arc};

use crate::federation::{DockerSubgraph, MockSubgraph};
use cynic_introspection::{IntrospectionQuery, SpecificationVersion};
use futures::{future::BoxFuture, stream::FuturesUnordered, StreamExt};
use graphql_mocks::MockGraphQlServer;
use url::Url;

#[derive(Clone)]
pub struct Subgraphs(Arc<Vec<Subgraph>>);

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
        mut others: Vec<Subgraph>,
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

        let (mut subgraphs, mut docker_subgraphs) = futures::join!(mock_subgraphs_fut, docker_subgraphs_fut);
        subgraphs.append(&mut docker_subgraphs);
        subgraphs.append(&mut others);

        // Ensures consistency of composition and thus introspection tests.
        subgraphs.sort_unstable_by(|a, b| a.name().cmp(b.name()));

        Self(Arc::new(subgraphs))
    }
}

pub enum Subgraph {
    Mock { type_id: TypeId, server: MockSubgraph },
    Docker { subgraph: DockerSubgraph, sdl: String },
    Virtual { name: String, sdl: String },
}

impl Subgraph {
    pub fn name(&self) -> &str {
        match self {
            Subgraph::Mock { server, .. } => &server.name,
            Subgraph::Docker { subgraph, .. } => subgraph.name(),
            Subgraph::Virtual { name, .. } => name,
        }
    }

    pub fn sdl(&self) -> Cow<'_, str> {
        match self {
            Subgraph::Mock { server, .. } => server.sdl().into(),
            Subgraph::Docker { sdl, .. } => sdl.into(),
            Subgraph::Virtual { sdl, .. } => sdl.into(),
        }
    }

    pub fn url(&self) -> Option<Url> {
        match self {
            Subgraph::Mock { server, .. } => Some(server.url()),
            Subgraph::Docker { subgraph, .. } => Some(subgraph.url()),
            Subgraph::Virtual { .. } => None,
        }
    }

    pub fn websocket_url(&self) -> Option<Url> {
        match self {
            Subgraph::Mock { server, .. } => Some(server.websocket_url()),
            Subgraph::Docker { subgraph, .. } => Some(subgraph.url()),
            Subgraph::Virtual { .. } => None,
        }
    }
}
