use std::sync::Arc;

use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{AuthenticatedState, AuthenticatedSynergiaApiError},
        SynergiaApi,
    },
};

// NOTE: as needed change the definitions here then adjust the real one in the api to it

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch timetable")]
    TimetableFetchError(#[source] CacheComputeError),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeekStart(String);

impl WeekStart {
    pub fn new(_0: String) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Subject {
    pub name: String,
    pub short: String,
}

#[derive(Debug, Clone)]
pub struct Lesson {
    pub number: usize,
    pub subject: Subject,
    pub teacher_name: String,
    pub hour_from: String,
    pub hour_to: String,
    pub is_canceled: bool,
    pub is_substitution: bool,
}

pub type TimeBlock = Vec<Lesson>;

pub type TimeBlocks = Vec<TimeBlock>;

#[derive(Debug, Clone)]
pub struct Day {
    pub time_blocks: TimeBlocks,
    pub date: String,
}

pub type InnerTimetable = Vec<Day>;

#[derive(Debug, Clone)]
pub struct Timetable {
    pub inner_timetable: InnerTimetable,
    pub week_start: WeekStart,
}

impl Keyable<WeekStart> for Timetable {
    fn key(&self) -> WeekStart {
        self.week_start.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TimetableRepository {
    cache: AutoKeyedCache<WeekStart, Timetable>,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
}

impl TimetableRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn timetable(&self, week_start: WeekStart) -> Result<Timetable, Error> {
        self.cache
            .try_get_with(&week_start, async {
                let inner_timetable = self
                    .synergia_api
                    .fetch_timetable(week_start.clone())
                    .await?;
                Ok::<_, AuthenticatedSynergiaApiError>(Timetable {
                    inner_timetable,
                    week_start: week_start.clone(),
                })
            })
            .await
            .map_err(Error::TimetableFetchError)
    }
}
