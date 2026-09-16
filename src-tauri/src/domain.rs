use thiserror::Error;

use crate::{
    domain::{daily_timetable::DailyTimetable, subject_grades::SubjectGrades},
    repositories::AppRepositories,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("daily timetable usecase failed")]
    DailyTimetableError(#[from] daily_timetable::Error),
    #[error("list subject grades usecase failed")]
    SubjectGradesError(#[from] subject_grades::Error),
}

pub mod daily_timetable;
pub mod subject_grades;

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

    pub async fn subject_grades_list(&self) -> Result<Vec<SubjectGrades>, Error> {
        Ok(subject_grades::subject_grades_list_usecase(&self.app_repos).await?)
    }
}
