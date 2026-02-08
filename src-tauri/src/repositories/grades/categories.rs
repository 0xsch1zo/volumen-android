use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{AuthenticatedState, AuthenticatedSynergiaApiError},
        SynergiaApi,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("category with id: {0:?}, not found")]
    CategoryNotFound(CategoryId),
    #[error("failed to fetch categories")]
    CategoryFetchFailed(#[source] CacheComputeError),
}

#[derive(Serialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(transparent)]
pub struct CategoryId(usize);

impl CategoryId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub count_to_the_average: bool,
    pub weight: Option<usize>,
}

impl Keyable<CategoryId> for Category {
    fn key(&self) -> CategoryId {
        self.id
    }
}

pub type Categories = Vec<Category>;

#[derive(Debug, Clone)]
pub struct CategoriesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CategoryId, Category>,
}

impl CategoriesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn category(&self, id: CategoryId) -> Result<Category, Error> {
        if let Some(category) = self.cache.get(&id).await {
            return Ok(category);
        }

        self.cache
            .try_bulk_insert_with(async {
                let categories = self.synergia_api.grades().fetch_categories().await?;
                Ok::<_, AuthenticatedSynergiaApiError>(categories)
            })
            .await
            .map_err(|e| Error::CategoryFetchFailed(e))?;

        Ok(self
            .cache
            .get(&id)
            .await
            .ok_or(Error::CategoryNotFound(id))?
            .clone())
    }
}
