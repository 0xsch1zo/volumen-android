use std::sync::Arc;

use crate::{
    cache::{Cache, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
    repositories::entities::Reference,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("subject with id: {0:?}, not found")]
    SubjectNotFound(SubjectId),
    #[error("subject fetch failed")]
    SubjectFetchFailed(#[source] synergia_api::Error),
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub struct SubjectId(usize);

impl From<Reference> for SubjectId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")] // what the fuck is this? Why is this in camel and the rest
                                   // is in pascal?
pub struct Subject {
    #[serde(rename = "numericIdentifier")]
    pub id: SubjectId,
    pub name: String,
}

impl Keyable<SubjectId> for Subject {
    fn key(&self) -> SubjectId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subjects {
    #[serde(rename = "data")]
    pub inner: Vec<Subject>,
}

#[derive(Debug, Clone)]
pub struct SubjectsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<Cache>,
}

impl SubjectsRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>, cache: Arc<Cache>) -> Self {
        Self {
            synergia_api,
            cache,
        }
    }

    pub async fn subject(&self, id: SubjectId) -> Result<Subject, Error> {
        if let Some(subject) = self.cache.subjects.read().await.get(&id) {
            return Ok(subject.clone());
        }

        let subjects = self
            .synergia_api
            .subjects()
            .await
            .map_err(|e| Error::SubjectFetchFailed(e))?;
        self.cache.subjects.write_values(subjects.inner).await;

        Ok(self
            .cache
            .subjects
            .read()
            .await
            .get(&id)
            .ok_or(Error::SubjectNotFound(id))?
            .clone())
    }
}
