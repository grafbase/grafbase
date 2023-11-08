use serde_json::json;
use url::Url;

#[derive(Debug, serde::Deserialize)]
struct Error {
    message: String,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    data: Option<serde_json::Value>,
    errors: Option<Vec<Error>>,
}

pub async fn add(name: &str, url: &Url, dev_api_port: u16, headers: Vec<(&str, &str)>) -> Result<(), crate::Error> {
    let headers = headers
        .into_iter()
        .map(|(key, value)| format!(r#"{{ key: "{key}", value: "{value}" }}"#))
        .collect::<Vec<_>>()
        .join(", ");

    let mutation = indoc::formatdoc! {r#"
        mutation {{
          publishSubgraph(input: {{
            name: "{name}"
            url: "{url}",
            headers: [{headers}]
          }}) {{ typename }}
        }}
    "#};

    let request = json!({
        "query": mutation,
        "variables": {}
    });

    let url = format!("http://localhost:{dev_api_port}/admin");

    let response: Response = reqwest::Client::new()
        .post(&url)
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    match (response.data, response.errors) {
        (Some(_), _) => Ok(()),
        (_, Some(errors)) => {
            let errors = errors
                .into_iter()
                .map(|error| error.message)
                .collect::<Vec<_>>()
                .join(", ");

            Err(crate::Error::internal(errors))
        }
        _ => unreachable!(),
    }
}
