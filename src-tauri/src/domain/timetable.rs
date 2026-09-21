use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use itertools::Itertools;
use serde::Serialize;
use thiserror::Error;

use crate::repositories::{
    self,
    calendar::{Month, Year},
    events::Event,
    timetable::TimeBlock,
    AppRepositories,
};

pub mod daily_timetable;
pub mod full_timetable;

// Private error type for shared functions in the timetable module
#[derive(Error, Debug)]
pub enum Error {
    #[error("event fetch error")]
    EventFetchError(#[source] repositories::events::Error),
    #[error("failed to fetch calendar")]
    CalendarFetchError(#[source] repositories::calendar::Error),
    #[error("failed to parse lesson time")]
    TimeBlockTimeParseError(#[source] chrono::ParseError),
    #[error("failed to parse lesson time")]
    EventTimeParseError(#[source] chrono::ParseError),
}

const YMD_FORMAT: &str = "%Y-%m-%d";

trait WeekOpExt {
    fn is_weekend(&self) -> bool;

    fn week_start(&self) -> Self;
}

impl WeekOpExt for NaiveDate {
    fn is_weekend(&self) -> bool {
        self.weekday().number_from_monday() >= Weekday::Sat.number_from_monday()
    }

    fn week_start(&self) -> NaiveDate {
        self.week(Weekday::Mon).first_day()
    }
}

enum EventTimeblockMatchingStatus {
    Matching,
    NotMatching,
    EmptyTimeblock,
}

impl EventTimeblockMatchingStatus {
    fn check(event: &Event, time_block: &TimeBlock) -> Result<Self, Error> {
        let time_from = NaiveTime::parse_from_str(&event.time_from, "%H:%M:%S")
            .map_err(Error::EventTimeParseError)?;
        let time_to = NaiveTime::parse_from_str(&event.time_to, "%H:%M:%S")
            .map_err(Error::EventTimeParseError)?;
        let status = time_block
            .first()
            .map(|t| {
                Ok::<_, chrono::ParseError>((
                    NaiveTime::parse_from_str(&t.hour_from, "%H:%M")?,
                    NaiveTime::parse_from_str(&t.hour_to, "%H:%M")?,
                ))
            })
            .transpose()
            .map_err(Error::TimeBlockTimeParseError)?
            .map(|(timeblock_time_from, timeblock_time_to)| {
                match time_from == timeblock_time_from && time_to == timeblock_time_to {
                    true => Self::Matching,
                    false => Self::NotMatching,
                }
            })
            .unwrap_or(Self::EmptyTimeblock);
        Ok(status)
    }
}

#[derive(Serialize, Debug)]
struct SubjectEventTimeBlock {
    start: String,
    end: String,
    subject: String,
    events: Vec<Event>,
}

impl SubjectEventTimeBlock {
    fn merge_from(
        time_block: TimeBlock,
        events: &Vec<Event>,
        date: NaiveDate,
    ) -> Result<Option<Self>, Error> {
        let filtered_events = events
            .iter()
            .filter(|e| e.date == date.format(YMD_FORMAT).to_string())
            .filter_map(|e| -> Option<Result<&Event, Error>> {
                let status = match EventTimeblockMatchingStatus::check(e, &time_block) {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e)),
                };
                match status {
                    EventTimeblockMatchingStatus::Matching => Some(Ok(e)),
                    EventTimeblockMatchingStatus::EmptyTimeblock => Some(Ok(e)),
                    EventTimeblockMatchingStatus::NotMatching => None,
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .cloned()
            .collect_vec();

        let Some(lesson) = time_block.first() else {
            return Ok(None);
        };

        Ok(Some(SubjectEventTimeBlock {
            start: lesson.hour_from.clone(),
            end: lesson.hour_to.clone(),
            subject: time_block.iter().map(|l| &l.subject.name).join(" | "),
            events: filtered_events,
        }))
    }
}

async fn fetch_events(
    app_repos: &AppRepositories,
    timetable_date: NaiveDate,
) -> Result<Vec<Event>, Error> {
    let calendar = app_repos
        .calendar()
        .calendar(
            Year::new(timetable_date.year()),
            Month::new(timetable_date.month() as u8),
        )
        .await
        .map_err(Error::CalendarFetchError)?;

    app_repos
        .events()
        .fetch_from_calendar(calendar)
        .await
        .map_err(Error::EventFetchError)
}

fn trim_timetable(
    time_blocks: Vec<Option<SubjectEventTimeBlock>>,
) -> Vec<Option<SubjectEventTimeBlock>> {
    time_blocks
        .into_iter()
        .skip_while(|t| t.is_none())
        .collect_vec()
        .into_iter()
        .rev()
        .skip_while(|t| t.is_none())
        .collect_vec()
        .into_iter()
        .rev()
        .collect()
}
