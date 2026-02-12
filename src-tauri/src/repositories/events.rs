use std::sync::Arc;

use thiserror::Error;

use crate::{
    cache::{CacheComputeError, SingleEntryCache},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        subjects::{Subject, SubjectId},
        users::{User, UserId},
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch event list")]
    EventListFetchError(#[source] CacheComputeError),
}

#[derive(Debug, Clone)]
pub struct EventId(usize);

impl EventId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct CategoryId(usize);

impl CategoryId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

pub struct Category {
    pub id: CategoryId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ShallowEvent {
    pub id: EventId,
    pub content: String,
    pub date: String,
    pub category: CategoryId,
    pub time_from: String,
    pub time_to: String,
    pub created_by: UserId,
    pub subject: Option<SubjectId>,
    pub add_date: String,
}

pub struct Event {
    pub id: EventId,
    pub content: String,
    pub date: String,
    pub category: Category,
    pub time_from: String,
    pub time_to: String,
    pub created_by: User,
    pub subject: Option<Subject>,
    pub add_date: String,
}

#[derive(Debug)]
pub struct EventsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: SingleEntryCache<Vec<ShallowEvent>>,
}

impl EventsRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: SingleEntryCache::new(),
        }
    }

    pub async fn list(&self) -> Result<Vec<ShallowEvent>, Error> {
        self.cache
            .try_get_with(async { self.synergia_api.events().list().await })
            .await
            .map_err(Error::EventListFetchError)
    }
}
