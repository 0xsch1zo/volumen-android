use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        self,
        events::EventId,
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
pub struct Year(i32);

impl Year {
    pub fn new(_0: i32) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> i32 {
        self.0
    }
}

#[derive(Serialize, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Month(u8);

impl Month {
    pub fn into_inner(self) -> u8 {
        self.0
    }
}

impl Month {
    pub fn new(_0: u8) -> Self {
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
    pub events: Vec<EventId>,
}

#[derive(Serialize, Debug, Clone)]
pub struct KeyedCalendar {
    pub key: CalendarKey,
    pub calendar: Calendar,
}

impl Keyable<CalendarKey> for KeyedCalendar {
    fn key(&self) -> CalendarKey {
        self.key.clone()
    }
}

#[derive(Debug, Clone)]
pub struct CalendarRepository {
    me_repo: MeRepository,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CalendarKey, KeyedCalendar>,
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

    pub async fn calendar(&self, year: Year, month: Month) -> Result<Calendar, Error> {
        let me = self.me_repo.me().await.map_err(Error::MeFetchError)?;
        let key = CalendarKey {
            class: me.class,
            year,
            month,
        };
        self.cache
            .try_get_with(&key, async {
                self.synergia_api
                    .fetch_calendar(me.class, year, month)
                    .await
                    .map(|calendar| KeyedCalendar {
                        key: key.clone(),
                        calendar,
                    })
            })
            .await
            .map(|c| c.calendar)
            .map_err(Error::CalendarFetchError)
    }
}
