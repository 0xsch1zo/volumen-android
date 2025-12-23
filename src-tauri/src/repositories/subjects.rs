use std::sync::Arc;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("subject with id: {0:?}, not found")]
    SubjectNotFound(SubjectId),
    #[error("subject fetch failed")]
    SubjectFetchFailed(#[source] CacheComputeError),
}

#[derive(Serialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(transparent)]
pub struct SubjectId(usize);

impl SubjectId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Subject {
    pub id: SubjectId,
    pub name: String,
    pub short: String,
    pub is_extracurricular: bool,
}

impl Keyable<SubjectId> for Subject {
    fn key(&self) -> SubjectId {
        self.id
    }
}

pub type Subjects = Vec<Subject>;

#[derive(Debug, Clone)]
pub struct SubjectsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<SubjectId, Subject>,
}

impl SubjectsRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn subject(&self, id: SubjectId) -> Result<Subject, Error> {
        if let Some(subject) = self.cache.get(&id).await {
            return Ok(subject);
        }

        self.cache
            .try_bulk_insert_with(async {
                let subjects = self.synergia_api.fetch_subjects().await?;
                Ok::<_, synergia_api::Error>(subjects)
            })
            .await
            .map_err(|e| Error::SubjectFetchFailed(e))?;

        Ok(self
            .cache
            .get(&id)
            .await
            .ok_or(Error::SubjectNotFound(id))?
            .clone())
    }
}
