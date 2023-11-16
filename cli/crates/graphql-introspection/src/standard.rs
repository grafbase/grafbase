use cynic::{http::ReqwestExt, QueryBuilder};
use cynic_introspection::IntrospectionQuery;
use reqwest::header::USER_AGENT;

pub(super) async fn introspect(url: &str, headers: &[(impl AsRef<str>, impl AsRef<str>)]) -> Result<String, String> {
    let mut request_builder = reqwest::Client::new().post(url).header(USER_AGENT, "Grafbase");

    for (name, value) in headers {
        request_builder = request_builder.header(name.as_ref(), value.as_ref());
    }

    let result = match request_builder.run_graphql(IntrospectionQuery::build(())).await {
        Ok(result) => match (result.errors, result.data) {
            (Some(errors), _) => {
                let message = errors
                    .into_iter()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                return Err(message);
            }
            (_, Some(data)) => data,
            _ => return Err(String::from("missing introspection data")),
        },
        Err(error) => return Err(error.to_string()),
    };

    let schema = match result.into_schema() {
        Ok(schema) => schema,
        Err(error) => return Err(error.to_string()),
    };

    Ok(schema.to_sdl())
}
