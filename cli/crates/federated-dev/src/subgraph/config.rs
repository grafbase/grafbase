use std::collections::BTreeMap;

use url::Url;

#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct SubgraphConfig {
    federation_version: String,
    subgraphs: BTreeMap<String, Subgraph>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct Subgraph {
    schema: Schema,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct Schema {
    subgraph_url: Url,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use url::Url;

    use crate::subgraph::config::{Subgraph, SubgraphConfig};

    #[test]
    fn can_parse_config() {
        let input = indoc::indoc! {r#"
            federation_version = "=2.6.0"

            [subgraphs.products]
            schema.subgraph_url = "http://localhost:4001/graphql"

            [subgraphs.users]
            schema.subgraph_url = "http://localhost:4002/graphql"
        "#};

        let mut subgraphs = BTreeMap::new();

        subgraphs.insert(
            String::from("products"),
            Subgraph {
                schema: crate::subgraph::config::Schema {
                    subgraph_url: Url::parse("http://localhost:4001/graphql").unwrap(),
                },
            },
        );

        subgraphs.insert(
            String::from("users"),
            Subgraph {
                schema: crate::subgraph::config::Schema {
                    subgraph_url: Url::parse("http://localhost:4002/graphql").unwrap(),
                },
            },
        );

        let expected = SubgraphConfig {
            federation_version: "=2.6.0".into(),
            subgraphs,
        };

        let config: SubgraphConfig = toml::from_str(input).unwrap();

        assert_eq!(expected, config);
    }
}
