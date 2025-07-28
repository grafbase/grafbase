use futures::future::BoxFuture;

use crate::gateway::TestRequest;

pub struct IntrospectionRequest(TestRequest);

impl From<TestRequest> for IntrospectionRequest {
    fn from(request: TestRequest) -> Self {
        Self(request)
    }
}

impl IntrospectionRequest {
    pub fn by_client(mut self, name: &'static str, version: &'static str) -> Self {
        self.0 = self.0.by_client(name, version);
        self
    }

    pub fn header<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.0 = self.0.header(name, value);
        self
    }

    pub fn header_append<Name, Value>(mut self, name: Name, value: Value) -> Self
    where
        Name: TryInto<http::HeaderName, Error: std::fmt::Debug>,
        Value: TryInto<http::HeaderValue, Error: std::fmt::Debug>,
    {
        self.0 = self.0.header_append(name, value);
        self
    }

    pub fn variables(mut self, variables: impl serde::Serialize) -> Self {
        self.0 = self.0.variables(variables);
        self
    }

    pub fn extensions(mut self, extensions: impl serde::Serialize) -> Self {
        self.0 = self.0.extensions(extensions);
        self
    }
}

impl IntoFuture for IntrospectionRequest {
    type Output = String;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let response = self.0.await;
            serde_json::from_value::<cynic_introspection::IntrospectionQuery>(response.into_data())
                .expect("valid response")
                .into_schema()
                .expect("valid schema")
                .to_sdl()
        })
    }
}
