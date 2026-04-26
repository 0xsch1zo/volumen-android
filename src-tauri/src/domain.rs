use thiserror::Error;

use crate::{domain::daily_timetable::DailyTimetable, repositories::AppRepositories};

#[derive(Error, Debug)]
pub enum Error {
    #[error("daily timetable usecase failed")]
    DailyTimetableError(#[from] daily_timetable::Error),
}

pub mod daily_timetable;

#[derive(Debug)]
pub struct AppUseCases {
    app_repos: AppRepositories,
}

impl AppUseCases {
    pub fn new(app_repos: AppRepositories) -> Self {
        Self { app_repos }
    }

    pub async fn daily_timetable(&self) -> Result<DailyTimetable, Error> {
        Ok(daily_timetable::daily_timetable_usecase(&self.app_repos).await?)
    }
}
