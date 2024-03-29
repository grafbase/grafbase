use std::mem::take;

use config::latest::Config;

use crate::sources;

use super::BuildContext;

pub struct ExternalDataSources {
    pub graphql: sources::graphql::GraphqlEndpoints,
}

impl ExternalDataSources {
    pub(super) fn build(ctx: &mut BuildContext, config: &mut Config) -> Self {
        let endpoints = take(&mut config.graph.subgraphs)
            .into_iter()
            .enumerate()
            .map(|(index, subgraph)| {
                let subgraph_id = ctx.next_subgraph_id();
                let name = subgraph.name.into();
                let url = ctx
                    .urls
                    .insert(url::Url::parse(&ctx.strings[subgraph.url.into()]).expect("valid url"));
                match config.subgraph_configs.remove(&federated_graph::SubgraphId(index)) {
                    Some(config::latest::SubgraphConfig { websocket_url, headers }) => {
                        sources::graphql::GraphqlEndpoint {
                            name,
                            subgraph_id,
                            url,
                            websocket_url: websocket_url
                                .map(|url| ctx.urls.insert(url::Url::parse(&config[url]).expect("valid url"))),
                            headers: headers.into_iter().map(Into::into).collect(),
                        }
                    }

                    None => sources::graphql::GraphqlEndpoint {
                        name,
                        subgraph_id,
                        url,
                        websocket_url: None,
                        headers: Vec::new(),
                    },
                }
            })
            .collect();
        ExternalDataSources {
            graphql: sources::GraphqlEndpoints { endpoints },
        }
    }
}
