mod standard;
mod subgraph;

pub async fn introspect(url: &str, headers: &[(&str, &str)]) -> Result<String, String> {
    if let Ok(result) = subgraph::introspect(url, headers).await {
        return Ok(prettify(result));
    };

    standard::introspect(url, headers).await.map(prettify)
}

fn prettify(graphql: String) -> String {
    let parsed = apollo_parser::Parser::new(&graphql).parse().document();

    match apollo_encoder::Document::try_from(parsed) {
        Ok(encoded) => encoded.to_string(),
        Err(_) => graphql,
    }
}
