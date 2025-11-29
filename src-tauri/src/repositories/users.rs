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
    #[error("user with id: {0:?}, not found")]
    UserNotFound(UserId),
    #[error("failed to fetch users")]
    UserFetchError(#[source] synergia_api::Error),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(from = "Reference")]
pub struct UserId(usize);

impl From<Reference> for UserId {
    fn from(value: Reference) -> Self {
        Self(value.id)
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
    #[serde(rename = "users")]
    pub inner: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct UsersRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<Cache>,
}

impl UsersRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>, cache: Arc<Cache>) -> Self {
        Self {
            synergia_api,
            cache,
        }
    }

    pub async fn user(&self, id: UserId) -> Result<User, Error> {
        if let Some(user) = self.cache.users.read().await.get(&id) {
            return Ok(user.clone());
        }

        let users = self
            .synergia_api
            .users()
            .await
            .map_err(|e| Error::UserFetchError(e))?;
        self.cache.users.write_values(users.inner).await;

        Ok(self
            .cache
            .users
            .read()
            .await
            .get(&id)
            .ok_or(Error::UserNotFound(id))?
            .clone())
    }
}
