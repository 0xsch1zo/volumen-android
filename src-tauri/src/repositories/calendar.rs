use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        self,
        me::{ClassId, MeRepository},
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch calendar")]
    CalendarFetchError(#[source] CacheComputeError),
    #[error("failed to get user profile")]
    MeFetchError(#[source] repositories::me::Error),
}

#[derive(Serialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct EventId(usize);

impl EventId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Year(usize);

impl Year {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Serialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Month(usize);

impl Month {
    fn into_inner(self) -> usize {
        self.0
    }
}

impl Month {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct CalendarKey {
    class: ClassId,
    year: Year,
    month: Month,
}

#[derive(Serialize, Debug, Clone)]
pub struct Calendar {
    pub key: CalendarKey,
    pub events: Vec<EventId>,
}

impl Keyable<CalendarKey> for Calendar {
    fn key(&self) -> CalendarKey {
        self.key.clone()
    }
}

#[derive(Debug, Clone)]
pub struct CalendarRepository {
    me_repo: MeRepository,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CalendarKey, Calendar>,
}

impl CalendarRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        let me_repo = MeRepository::new(Arc::clone(&synergia_api));
        Self {
            me_repo,
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn calendar(
        &self,
        class: ClassId,
        year: Year,
        month: Month,
    ) -> Result<Calendar, Error> {
        let me = self.me_repo.me().await.map_err(Error::MeFetchError)?;
        let key = CalendarKey { class, year, month };
        self.cache
            .try_get_with(&key, async {
                self.synergia_api
                    .fetch_calendar(me.class, year, month)
                    .await
            })
            .await
            .map_err(Error::CalendarFetchError)
    }
}
