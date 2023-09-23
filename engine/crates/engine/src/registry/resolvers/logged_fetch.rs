use runtime::log::LogEvent;

pub async fn send_logged_request(
    request_id: &str,
    fetch_log_endpoint_url: Option<&str>,
    request_builder: reqwest::RequestBuilder,
) -> Result<http::Response<bytes::Bytes>, reqwest::Error> {
    let start_time = web_time::Instant::now();

    let (client, request) = request_builder.build_split();
    let request = request?;

    let url = request.url().to_string();
    let method = request.method().to_string();

    let mut response = client.execute(request).await?;
    let status_code = response.status().as_u16();

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::to_owned);

    let mut new_response = http::Response::builder().status(status_code);
    new_response
        .headers_mut()
        .unwrap()
        .extend(response.headers_mut().drain());

    let bytes = response.bytes().await?;

    if let Some(fetch_log_endpoint_url) = fetch_log_endpoint_url {
        let body = match content_type.as_deref() {
            Some("application/json" | "text/plain" | "text/html") => String::from_utf8(bytes.to_vec()).ok(),
            _ => None,
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
    }

    Ok(new_response.body(bytes).expect("must be valid"))
}
