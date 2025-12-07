use std::{fmt::Debug, future::Future, hash::Hash, sync::Arc};

use futures::stream::{self, StreamExt};
use moka::future::Cache;
use thiserror::Error;

use crate::sync::SingleParallelFlight;

#[derive(Error, Clone, Debug)]
#[error("an error occured while computing the value of a cache entry")]
pub struct CacheComputeError(#[source] Arc<dyn std::error::Error + Sync + Send + 'static>);

impl CacheComputeError {
    pub fn from_err<E: std::error::Error + Sync + Send + 'static>(err: E) -> Self {
        Self(Arc::new(err))
    }
}

pub trait Keyable<K: Copy + Hash + Eq> {
    fn key(&self) -> K;
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AutoKeyedCache<
    K: Send + Sync + Clone + Copy + Hash + Eq + 'static,
    V: Send + Sync + Clone + Keyable<K> + 'static,
> {
    cache: Cache<K, V>,
    get_with_worker: Arc<SingleParallelFlight<V>>,
    try_get_with_worker: Arc<SingleParallelFlight<Result<V, CacheComputeError>>>,
    bulk_insert_worker: Arc<SingleParallelFlight<()>>,
    try_bulk_insert_worker: Arc<SingleParallelFlight<Result<(), CacheComputeError>>>,
}

impl<
        K: Send + Sync + Clone + Copy + Hash + Eq + 'static,
        V: Send + Sync + Clone + Keyable<K> + 'static,
    > AutoKeyedCache<K, V>
{
    pub fn new() -> Self {
        Self {
            cache: Cache::builder().build(),
            get_with_worker: Arc::new(SingleParallelFlight::new()),
            try_get_with_worker: Arc::new(SingleParallelFlight::new()),
            bulk_insert_worker: Arc::new(SingleParallelFlight::new()),
            try_bulk_insert_worker: Arc::new(SingleParallelFlight::new()),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).await
    }

    pub async fn insert(&self, value: V) {
        self.cache.insert(value.key(), value).await;
    }

    // There is an implementation of try_get_with in moka already, but to provide consistent behaviour
    // we're rolling our own. What could possible go wrong?
    #[allow(unused)]
    pub async fn get_with(&self, key: &K, init: impl Future<Output = V>) -> V {
        if let Some(v) = self.get(key).await {
            return v;
        }
        self.get_with_worker
            .work(async || {
                let value = init.await;
                self.insert(value.clone()).await;
                value
            })
            .await
    }

    pub async fn try_get_with<E: std::error::Error + Send + Sync + 'static>(
        &self,
        key: &K,
        init: impl Future<Output = Result<V, E>>,
    ) -> Result<V, CacheComputeError> {
        if let Some(v) = self.get(key).await {
            return Ok(v);
        }
        let v = self
            .try_get_with_worker
            .work(async || {
                let value = init.await.map_err(CacheComputeError::from_err)?;
                self.insert(value.clone()).await;
                Ok(value)
            })
            .await?;
        Ok(v)
    }

    #[allow(unused)]
    pub async fn bulk_insert_with(&self, init: impl Future<Output = impl IntoIterator<Item = V>>) {
        self.bulk_insert_worker
            .work(async || {
                stream::iter(init.await.into_iter())
                    .for_each(async |v: V| {
                        self.cache.insert(v.key(), v).await;
                    })
                    .await;
            })
            .await;
    }

    pub async fn try_bulk_insert_with<E: std::error::Error + Send + Sync + 'static>(
        &self,
        init: impl Future<Output = Result<impl IntoIterator<Item = V>, E>>,
    ) -> Result<(), CacheComputeError> {
        self.try_bulk_insert_worker
            .work(async || {
                stream::iter(init.await.map_err(CacheComputeError::from_err)?.into_iter())
                    .for_each(async |v: V| {
                        self.cache.insert(v.key(), v).await;
                    })
                    .await;
                Ok(())
            })
            .await
    }
}
