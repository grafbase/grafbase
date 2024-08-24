use reqwest::header::USER_AGENT;
use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
struct Service {
    sdl: String,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    #[serde(rename = "_service")]
    service: Service,
}

#[derive(Debug, serde::Deserialize)]
struct Response {
    data: Option<Data>,
}

#[derive(Debug, serde::Serialize)]
struct Request {
    query: &'static str,
    variables: HashMap<&'static str, String>,
}

pub(super) async fn introspect(url: &str, headers: &[(impl AsRef<str>, impl AsRef<str>)]) -> Result<String, ()> {
    let query = indoc::indoc! {r"
        query {
          _service {
            sdl
          }
        }
    "};

    let request = Request {
        query,
        variables: HashMap::default(),
    };

    let mut request_builder = reqwest::Client::new()
        .post(url)
        .header(USER_AGENT, "Grafbase")
        .header("Accept", "application/json")
        .json(&request);

    for (name, value) in headers {
        request_builder = request_builder.header(name.as_ref(), value.as_ref());
    }

    let Ok(response) = request_builder.send().await else {
        return Err(());
    };

    let Ok(response) = response.error_for_status() else {
        return Err(());
    };

    let response: Response = match response.json().await {
        Ok(response) => response,
        Err(_) => return Err(()),
    };

    match response.data {
        Some(data) => Ok(data.service.sdl),
        _ => Err(()),
    }
}
