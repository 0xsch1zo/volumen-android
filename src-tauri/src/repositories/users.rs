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
    #[error("user with id: {0:?}, not found")]
    UserNotFound(UserId),
    #[error("failed to fetch users")]
    UserFetchError(#[source] CacheComputeError),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(from = "Reference")]
pub struct UserId(usize);

impl From<Reference> for UserId {
    fn from(value: Reference) -> Self {
        Self(match value {
            Reference::Linked { id, .. } => id,
            Reference::Standalone(id) => id,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub id: UserId,
    pub first_name: String,
    pub last_name: String,
}

impl Keyable<UserId> for User {
    fn key(&self) -> UserId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Users {
    #[serde(rename = "Users")]
    pub inner: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct UsersRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<UserId, User>,
}

impl UsersRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn user(&self, id: UserId) -> Result<User, Error> {
        if let Some(user) = self.cache.get(&id).await {
            return Ok(user);
        }

        self.cache
            .try_bulk_insert_with(async {
                Ok::<_, synergia_api::Error>(self.synergia_api.users().await?.inner)
            })
            .await
            .map_err(|e| Error::UserFetchError(e))?;

        Ok(self
            .cache
            .get(&id)
            .await
            .ok_or(Error::UserNotFound(id))?
            .clone())
    }
}
