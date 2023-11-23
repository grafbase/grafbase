use runtime::log::LogEvent;

pub async fn send_logged_request(
    request_id: &str,
    fetch_log_endpoint_url: Option<&str>,
    client: &reqwest::Client,
    request_builder: reqwest::RequestBuilder,
) -> Result<reqwest::Response, reqwest::Error> {
    let start_time = web_time::Instant::now();

    let request = request_builder.build()?;

    let url = request.url().to_string();
    let method = request.method().to_string();

    let mut response = client.execute(request).await?;

    if let Some(fetch_log_endpoint_url) = fetch_log_endpoint_url {
        let status_code = response.status().as_u16();

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::to_owned);

        let (response_to_return, body) = match content_type.as_deref() {
            Some("application/json" | "text/plain" | "text/html") => {
                let mut response_to_return_builder = http::Response::builder().status(status_code);
                response_to_return_builder
                    .headers_mut()
                    .unwrap()
                    .extend(response.headers_mut().drain());
                let bytes = response.bytes().await?;
                let body = String::from_utf8(bytes.to_vec()).ok();
                let response_to_return = response_to_return_builder.body(bytes).expect("must be valid").into();
                (response_to_return, body)
            }
            _ => (response, None),
        };

        let duration = start_time.elapsed();
        reqwest::Client::new()
            .post(format!("{fetch_log_endpoint_url}/log-event"))
            .json(&LogEvent {
                request_id,
                r#type: common_types::LogEventType::NestedRequest {
                    url,
                    method,
                    status_code,
                    duration,
                    body,
                    content_type,
                },
            })
            .send()
            .await?;

        Ok(response_to_return)
    } else {
        Ok(response)
    }
}
