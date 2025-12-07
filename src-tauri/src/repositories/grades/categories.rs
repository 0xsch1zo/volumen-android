use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
    repositories::entities::Reference,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("category with id: {0:?}, not found")]
    CategoryNotFound(CategoryId),
    #[error("failed to fetch categories")]
    CategoryFetchFailed(#[source] CacheComputeError),
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub struct CategoryId(usize);

impl From<Reference> for CategoryId {
    fn from(value: Reference) -> Self {
        Self(match value {
            Reference::Linked { id, .. } => id,
            Reference::Standalone(id) => id,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Categories {
    #[serde(rename = "Categories")]
    pub inner: Vec<Category>,
}

#[derive(Debug, Clone)]
pub struct CategoriesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<AutoKeyedCache<CategoryId, Category>>,
}

impl CategoriesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: Arc::new(AutoKeyedCache::new()),
        }
    }

    pub async fn category(&self, id: CategoryId) -> Result<Category, Error> {
        if let Some(category) = self.cache.get(&id).await {
            return Ok(category);
        }

        self.cache
            .try_bulk_insert_with(async {
                let categories = self.synergia_api.categories().await?;
                Ok::<_, synergia_api::Error>(categories.inner)
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
