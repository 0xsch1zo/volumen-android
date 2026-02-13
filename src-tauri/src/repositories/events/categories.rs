use std::sync::Arc;

use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch category list")]
    CategoryListFetchError(#[source] CacheComputeError),
    #[error("category not found")]
    CategoryNotFound,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CategoryId(usize);

impl CategoryId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Clone, Hash)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
}

impl Keyable<CategoryId> for Category {
    fn key(&self) -> CategoryId {
        self.id
    }
}

#[derive(Debug)]
pub struct CategoriesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CategoryId, Category>,
}

impl CategoriesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            cache: AutoKeyedCache::new(),
            synergia_api,
        }
    }

    pub async fn category(&self, id: CategoryId) -> Result<Category, Error> {
        if let Some(category) = self.cache.get(&id).await {
            return Ok(category);
        }

        self.cache
            .try_bulk_insert_with(async { self.synergia_api.events().fetch_categories().await })
            .await
            .map_err(Error::CategoryListFetchError)?;

        self.cache.get(&id).await.ok_or(Error::CategoryNotFound)
    }
}
