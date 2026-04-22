use std::sync::Arc;

use thiserror::Error;
use tokio::sync::mpsc;

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

// TODO: this has to change, we cannot require the frontend to figure out the date of the start of
// the week
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Date(String);

impl Date {
    pub fn new(_0: String) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Clone)]
pub struct Day {
    pub time_blocks: TimeBlocks,
    pub date: Date,
}

impl Keyable<Date> for Day {
    fn key(&self) -> Date {
        self.date.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Timetable {
    pub timetable: Vec<Day>,
}

#[derive(Clone, Debug)]
pub struct TimetableRepository {
    cache: AutoKeyedCache<Date, Day>,
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
        let (tx, mut rx) = mpsc::channel(1);
        self.cache
            .try_bulk_insert_with(async {
                let timetable = self.synergia_api.fetch_timetable(week_start).await?;
                tx.send(timetable.clone()).await.unwrap();
                Ok::<_, AuthenticatedSynergiaApiError>(timetable.timetable)
            })
            .await
            .map_err(Error::TimetableFetchError)?;
        Ok(rx.recv().await.unwrap())
    }
}
