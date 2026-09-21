use chrono::{Local, NaiveDate};
use serde::Serialize;
use thiserror::Error;

use crate::{
    domain::timetable::{SubjectEventTimeBlock, WeekOpExt, YMD_FORMAT},
    repositories::{self, timetable::WeekStart, AppRepositories},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse timetable date")]
    TimetableDateParseError(#[source] chrono::ParseError),
    #[error("failed to fetch timetable")]
    TimetableFetchError(#[source] repositories::timetable::Error),
    #[error("failed to fetch events")]
    EventFetchError(#[source] super::Error),
    #[error("failed to merge a timeblock and events to construct a subject-event timeblock")]
    SubjectEventTimeblockMergeError(super::Error),
}

#[derive(Debug, Serialize)]
pub struct Timetable {
    date: String,
    days: Vec<Day>,
}

#[derive(Debug, Serialize)]
pub struct Day {
    date: String,
    time_blocks: Vec<Option<SubjectEventTimeBlock>>,
}

pub async fn full_timetable_usecase(
    app_repos: &AppRepositories,
    date: Option<String>,
) -> Result<Timetable, Error> {
    let date = match date {
        Some(date) => {
            NaiveDate::parse_from_str(&date, YMD_FORMAT).map_err(Error::TimetableDateParseError)?
        }
        None => Local::now().date_naive(),
    };

    let timetable = app_repos
        .timetables()
        .timetable(WeekStart::new(
            date.week_start().format(YMD_FORMAT).to_string(),
        ))
        .await
        .map_err(Error::TimetableFetchError)?;

    let events = super::fetch_events(app_repos, date)
        .await
        .map_err(Error::EventFetchError)?;

    let days = timetable
        .inner_timetable
        .into_iter()
        .map(|day| -> Result<Day, Error> {
            let date = NaiveDate::parse_from_str(&day.date, YMD_FORMAT)
                .map_err(Error::TimetableDateParseError)?;
            let time_blocks = day
                .time_blocks
                .into_iter()
                .map(|time_block| SubjectEventTimeBlock::merge_from(time_block, &events, date))
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::SubjectEventTimeblockMergeError)?;
            Ok(Day {
                date: day.date,
                time_blocks,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Timetable {
        date: date.format(YMD_FORMAT).to_string(),
        days,
    })
}
