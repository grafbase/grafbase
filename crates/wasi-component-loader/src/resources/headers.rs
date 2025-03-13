use std::sync::Arc;

use crate::extension::api::wit::HeaderError;

pub enum Headers {
    Shared(Arc<http::HeaderMap>),
    SharedMut(Arc<tokio::sync::RwLock<http::HeaderMap>>),
    /// Borrowed from the host, the instance will be provided with T, but it should not be dropped.
    /// The caller will remove it himself from the store.
    Borrow(http::HeaderMap),
    /// Fully owned by the guest
    Owned(http::HeaderMap),
}

impl Headers {
    pub fn is_owned(&self) -> bool {
        matches!(self, Headers::Owned(_))
    }

    pub fn unborrow(self) -> Option<http::HeaderMap> {
        match self {
            Headers::Borrow(headers) => Some(headers),
            _ => None,
        }
    }

    pub async fn get(&self, name: &str) -> Vec<Vec<u8>> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(headers) => headers.as_ref(),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.read().await);
                _guard.as_deref().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        headers
            .get_all(name)
            .into_iter()
            .map(|val| val.as_bytes().to_vec())
            .collect()
    }

    pub async fn has(&self, name: &str) -> bool {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(headers) => headers.as_ref(),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.read().await);
                _guard.as_deref().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        headers.contains_key(name)
    }

    pub async fn set(&mut self, name: String, value: Vec<Vec<u8>>) -> Result<(), HeaderError> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(_) => return Err(HeaderError::Immutable),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.write().await);
                _guard.as_deref_mut().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        if value.len() == 1 {
            headers.insert(
                name,
                value
                    .into_iter()
                    .next()
                    .unwrap()
                    .try_into()
                    .map_err(|_| HeaderError::InvalidSyntax)?,
            );
        } else {
            headers.remove(&name);
            for value in value {
                headers.append(name.clone(), value.try_into().map_err(|_| HeaderError::InvalidSyntax)?);
            }
        }
        Ok(())
    }

    pub async fn delete(&mut self, name: &str) -> Result<(), HeaderError> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(_) => return Err(HeaderError::Immutable),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.write().await);
                _guard.as_deref_mut().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        headers.remove(name);
        Ok(())
    }

    pub async fn get_and_delete(&mut self, name: &str) -> Result<Vec<Vec<u8>>, HeaderError> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(_) => return Err(HeaderError::Immutable),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.write().await);
                _guard.as_deref_mut().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        match headers.entry(name) {
            http::header::Entry::Occupied(entry) => {
                let (_, values) = entry.remove_entry_mult();
                Ok(values.into_iter().map(|val| val.as_bytes().to_vec()).collect())
            }
            http::header::Entry::Vacant(_) => Ok(Vec::new()),
        }
    }

    pub async fn append(&mut self, name: String, value: Vec<u8>) -> Result<(), HeaderError> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(_) => return Err(HeaderError::Immutable),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.write().await);
                _guard.as_deref_mut().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
        headers.append(name, value.try_into().map_err(|_| HeaderError::InvalidSyntax)?);
        Ok(())
    }

    pub async fn entries(&self) -> Vec<(String, Vec<u8>)> {
        let mut _guard = None;
        let headers = match self {
            Headers::Shared(headers) => headers.as_ref(),
            Headers::SharedMut(headers) => {
                _guard = Some(headers.read().await);
                _guard.as_deref().unwrap()
            }
            Headers::Borrow(headers) => headers,
            Headers::Owned(headers) => headers,
        };
        headers
            .iter()
            .map(|(name, values)| {
                let name = name.as_str().to_string();
                let values = values.as_bytes().to_vec();
                (name, values)
            })
            .collect()
    }
}
