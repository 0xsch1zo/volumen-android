use std::{collections::HashMap, fmt::Debug, hash::Hash};

use thiserror::Error;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::repositories::{
    categories::{Category, CategoryId},
    subjects::{Subject, SubjectId},
    users::{User, UserId},
};

#[derive(Error, Debug)]
#[error("cache entry not found")]
struct CacheEntryNotFoudError;

#[derive(Debug)]
pub struct Cache {
    pub users: KeyedCacheResource<UserId, User>,
    pub subjects: KeyedCacheResource<SubjectId, Subject>,
    pub categories: KeyedCacheResource<CategoryId, Category>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            users: KeyedCacheResource::new(),
            subjects: KeyedCacheResource::new(),
            categories: KeyedCacheResource::new(),
        }
    }
}

pub trait Keyable<K: Copy + Hash + Eq> {
    fn key(&self) -> K;
}

#[derive(Debug)]
struct KeyedCacheResource<K: Copy + Hash + Eq, V: Keyable<K>> {
    resource: RwLock<HashMap<K, V>>,
}

impl<K: Copy + Hash + Eq, V: Keyable<K>> KeyedCacheResource<K, V> {
    fn new() -> Self {
        Self {
            resource: RwLock::new(HashMap::new()),
        }
    }

    pub async fn read(&self) -> RwLockReadGuard<HashMap<K, V>> {
        self.resource.read().await
    }

    pub async fn write(&self, key: K, value: V) -> Result<(), CacheEntryNotFoudError> {
        self.resource
            .write()
            .await
            .insert(key, value)
            .map(|_| ())
            .ok_or(CacheEntryNotFoudError)
    }

    pub async fn write_values<C: IntoIterator<Item = V>>(&self, new_values: C) {
        let new_resource = new_values
            .into_iter()
            .map(|r| (r.key(), r))
            .collect::<HashMap<_, _>>();
        let mut resource_lock = self.resource.write().await;
        *resource_lock = new_resource;
    }
}
