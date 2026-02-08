use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState, AuthenticatedSynergiaApiError},
        SynergiaApi,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("user with id: {0:?}, not found")]
    UserNotFound(UserId),
    #[error("failed to fetch users")]
    UserFetchError(#[source] CacheComputeError),
}

#[derive(Serialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(transparent)]
pub struct UserId(usize);

impl UserId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub first_name: String,
    pub last_name: String,
}

pub type Users = Vec<User>;

impl Keyable<UserId> for User {
    fn key(&self) -> UserId {
        self.id
    }
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
                Ok::<_, AuthenticatedSynergiaApiError>(self.synergia_api.fetch_users().await?)
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
