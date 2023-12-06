use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, Fetcher, FetcherInner};

pub struct NativeFetcher {
    client: reqwest::Client,
}

impl NativeFetcher {
    pub fn runtime_fetcher() -> Fetcher {
        Fetcher::new(Box::new(Self {
            client: reqwest::Client::new(),
        }))
    }
}

#[async_trait::async_trait]
impl FetcherInner for NativeFetcher {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        let response = self
            .client
            .post(request.url)
            .body(request.json_body)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| FetchError::AnyError(e.to_string()))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| FetchError::AnyError(e.to_string()))?;
        Ok(FetchResponse { bytes })
    }
}
