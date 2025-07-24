use crate::extension::api::since_0_17_0::world::HeaderError;

use super::OwnedOrShared;

pub enum WasmOwnedOrLease<T> {
    Owned(T),
    Lease(OwnedOrShared<T>),
}

impl<T> WasmOwnedOrLease<T> {
    pub(crate) fn into_inner(self) -> Option<T> {
        match self {
            Self::Owned(v) => Some(v),
            Self::Lease(lease) => lease.into_inner(),
        }
    }

    pub(crate) fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    pub(crate) fn into_lease(self) -> Option<OwnedOrShared<T>> {
        match self {
            Self::Lease(v) => Some(v),
            _ => None,
        }
    }

    pub(crate) async fn with_ref<R>(&self, f: impl FnOnce(&T) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Self::Lease(OwnedOrShared::Shared(v)) => v.as_ref(),
            Self::Lease(OwnedOrShared::SharedMut(v)) => {
                _guard = Some(v.read().await);
                _guard.as_deref().unwrap()
            }
            Self::Lease(OwnedOrShared::Owned(v)) => v,
            Self::Owned(v) => v,
        };
        f(v)
    }

    pub(crate) async fn with_ref_mut<R>(&mut self, f: impl FnOnce(Option<&mut T>) -> R) -> R
    where
        T: Send + Sync + 'static,
    {
        let mut _guard = None;
        let v = match self {
            Self::Lease(OwnedOrShared::Shared(_)) => None,
            Self::Lease(OwnedOrShared::SharedMut(v)) => {
                _guard = Some(v.write().await);
                _guard.as_deref_mut()
            }
            Self::Lease(OwnedOrShared::Owned(v)) => Some(v),
            Self::Owned(v) => Some(v),
        };
        f(v)
    }
}

impl<T> From<T> for WasmOwnedOrLease<T> {
    fn from(v: T) -> Self {
        Self::Owned(v)
    }
}

impl<T> From<OwnedOrShared<T>> for WasmOwnedOrLease<T> {
    fn from(v: OwnedOrShared<T>) -> Self {
        Self::Lease(v)
    }
}

pub type LegacyHeaders = WasmOwnedOrLease<http::HeaderMap>;

impl LegacyHeaders {
    pub async fn get(&self, name: &str) -> Vec<Vec<u8>> {
        self.with_ref(|headers| {
            headers
                .get_all(name)
                .into_iter()
                .map(|val| val.as_bytes().to_vec())
                .collect()
        })
        .await
    }

    pub async fn has(&self, name: &str) -> bool {
        self.with_ref(|headers| headers.contains_key(name)).await
    }

    pub async fn set(&mut self, name: String, value: Vec<Vec<u8>>) -> Result<(), HeaderError> {
        self.with_ref_mut(|headers| {
            let Some(headers) = headers else {
                return Err(HeaderError::Immutable);
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
        })
        .await
    }

    pub async fn delete(&mut self, name: &str) -> Result<(), HeaderError> {
        self.with_ref_mut(|headers| {
            let Some(headers) = headers else {
                return Err(HeaderError::Immutable);
            };
            headers.remove(name);
            Ok(())
        })
        .await
    }

    pub async fn get_and_delete(&mut self, name: &str) -> Result<Vec<Vec<u8>>, HeaderError> {
        self.with_ref_mut(|headers| {
            let Some(headers) = headers else {
                return Err(HeaderError::Immutable);
            };
            let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
            match headers.entry(name) {
                http::header::Entry::Occupied(entry) => {
                    let (_, values) = entry.remove_entry_mult();
                    Ok(values.into_iter().map(|val| val.as_bytes().to_vec()).collect())
                }
                http::header::Entry::Vacant(_) => Ok(Vec::new()),
            }
        })
        .await
    }

    pub async fn append(&mut self, name: String, value: Vec<u8>) -> Result<(), HeaderError> {
        self.with_ref_mut(|headers| {
            let Some(headers) = headers else {
                return Err(HeaderError::Immutable);
            };
            let name: http::HeaderName = name.try_into().map_err(|_| HeaderError::InvalidSyntax)?;
            headers.append(name, value.try_into().map_err(|_| HeaderError::InvalidSyntax)?);
            Ok(())
        })
        .await
    }

    pub async fn entries(&self) -> Vec<(String, Vec<u8>)> {
        self.with_ref(|headers| {
            headers
                .iter()
                .map(|(name, values)| {
                    let name = name.as_str().to_string();
                    let values = values.as_bytes().to_vec();
                    (name, values)
                })
                .collect()
        })
        .await
    }
}
