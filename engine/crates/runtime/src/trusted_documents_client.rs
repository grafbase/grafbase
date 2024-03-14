pub struct Client(Box<dyn TrustedDocumentsClient>);

impl Client {
    pub fn new<T>(inner: T) -> Self
    where
        T: TrustedDocumentsClient + 'static,
    {
        Client(Box::new(inner))
    }
}

impl std::ops::Deref for Client {
    type Target = dyn TrustedDocumentsClient;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub enum TrustedDocumentsError {
    RetrievalError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DocumentNotFound,
}

pub type TrustedDocumentsResult<T> = Result<T, TrustedDocumentsError>;

/// A handle to trusted documents configuration and retrieval.
#[async_trait::async_trait]
pub trait TrustedDocumentsClient: Send + Sync {
    fn is_enabled(&self) -> bool;

    /// Users can optionally configure a header (name, value) which, when it is
    /// sent with a request, will bypass the trusted documents checks and allow running
    /// arbitrary queries.
    fn bypass_header(&self) -> Option<(&str, &str)> {
        None
    }

    async fn fetch(&self, client_name: &str, document_id: &str) -> TrustedDocumentsResult<String>;
}
