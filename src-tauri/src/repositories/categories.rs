use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cache::{Cache, Keyable},
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
    CategoryFetchFailed(#[source] synergia_api::Error),
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub struct CategoryId(usize);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Category {
    id: CategoryId,
    name: String,
    count_to_the_average: bool,
    weight: bool,
}

impl From<Reference> for CategoryId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
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

pub struct CategoriesRepository {
    cache: Arc<Cache>,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
}

impl CategoriesRepository {
    pub fn new(cache: Arc<Cache>, synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            cache,
            synergia_api,
        }
    }

    pub async fn category(&self, id: &CategoryId) -> Result<Category, Error> {
        if let Some(category) = self.cache.categories.read().await.get(id) {
            return Ok(category.clone());
        }

        let categories = self
            .synergia_api
            .categories()
            .await
            .map_err(|e| Error::CategoryFetchFailed(e))?;
        self.cache.categories.write_values(categories.inner).await;

        Ok(self
            .cache
            .categories
            .read()
            .await
            .get(&id)
            .ok_or(Error::CategoryNotFound(*id))?
            .clone())
    }
}
