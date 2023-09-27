use std::{marker::PhantomData, sync::Arc};

use runtime::cache::{Cache, Cacheable, Entry, EntryState, Result};

#[derive(Default)]
pub struct NoopCache<T> {
    _marker: PhantomData<T>,
}

impl<T> NoopCache<T> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync + Cacheable + 'static> Cache for NoopCache<T> {
    type Value = T;

    async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
        Ok(Entry::Miss)
    }

    async fn put(&self, _key: &str, _state: EntryState, _value: Arc<Self::Value>, _tags: Vec<String>) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    async fn purge_by_tags(&self, _tags: Vec<String>) -> Result<()> {
        Ok(())
    }

    async fn purge_by_hostname(&self, _hostname: String) -> Result<()> {
        Ok(())
    }
}
