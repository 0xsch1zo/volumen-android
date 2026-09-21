use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime, Weekday};
use serde::Serialize;
use thiserror::Error;

use crate::{
    domain::timetable::{SubjectEventTimeBlock, WeekOpExt, YMD_FORMAT},
    repositories::{
        self,
        timetable::{TimeBlocks, WeekStart},
        AppRepositories,
    },
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to count to get new date for next week")]
    NextWeekDateError,
    #[error("tomorrow date error")]
    TomorrowDateError,
    #[error("failed to parse lesson end time")]
    LessonEndTimeParseError(#[source] chrono::ParseError),
    #[error("timetable fetch error")]
    TimetableFetchError(#[source] repositories::timetable::Error),
    #[error("lessons for needed timetable date not found")]
    LessonsForDateNotFound,
    #[error("failed to fetch events")]
    EventFetchError(super::Error),
    #[error("failed to merge a timeblock and events to construct a subject-event timeblock")]
    SubjectEventTimeblockMergeError(super::Error),
}

#[derive(Serialize, Debug)]
enum TimetableWhen {
    Today,
    Tomorrow,
    NextWeek,
}

impl TimetableWhen {
    async fn fetch_from_current_timetable(
        app_repos: &AppRepositories,
        today: NaiveDate,
        current_time: NaiveTime,
    ) -> Result<Self, Error> {
        let current_week_timetable = app_repos
            .timetables()
            .timetable(WeekStart::new(
                today.week_start().format(YMD_FORMAT).to_string(),
            ))
            .await
            .map_err(Error::TimetableFetchError)?;

        let todays_lessons = current_week_timetable
            .inner_timetable
            .iter()
            .find(|day| day.date == today.format(YMD_FORMAT).to_string())
            .ok_or(Error::LessonsForDateNotFound)?;

        match (today.is_weekend(), todays_lessons.time_blocks.last()) {
            (true, _) => Ok(TimetableWhen::NextWeek),
            (false, None) => Ok(TimetableWhen::Tomorrow),
            (false, Some::<&repositories::timetable::TimeBlock>(last_timeblock))
                if !last_timeblock.is_empty() =>
            {
                let last_timeblock = last_timeblock
                    .iter()
                    .map(|lesson| NaiveTime::parse_from_str(&lesson.hour_to, "%H:%M"))
                    .collect::<Result<Vec<_>, chrono::ParseError>>()
                    .map_err(Error::LessonEndTimeParseError)?;

                let last_lesson_end_time = last_timeblock
                    .iter()
                    .max()
                    .expect("the last timeblock should be check for empty before accessing");
                if current_time <= *last_lesson_end_time {
                    Ok(TimetableWhen::Today)
                } else {
                    Ok(TimetableWhen::Tomorrow)
                }
            }
            _ => Ok(TimetableWhen::Tomorrow),
        }
    }

    fn week_start(&self, today: NaiveDate) -> Result<NaiveDate, Error> {
        Ok(match self {
            TimetableWhen::NextWeek => {
                let days_until_new_week = Weekday::Mon.days_since(today.weekday());
                today
                    .clone()
                    .checked_add_days(Days::new(days_until_new_week as u64))
                    .ok_or(Error::NextWeekDateError)?
            }
            _ => today.week_start(),
        })
    }
}

#[derive(Serialize, Debug)]
pub struct DailyTimetable {
    day_of_week: String,
    when: TimetableWhen,
    time_blocks: Vec<Option<SubjectEventTimeBlock>>,
}

async fn fetch_timeblocks_of_day(
    app_repos: &AppRepositories,
    week_start: NaiveDate,
    timetable_date: NaiveDate,
) -> Result<TimeBlocks, Error> {
    let timetable = app_repos
        .timetables()
        .timetable(WeekStart::new(week_start.format(YMD_FORMAT).to_string()))
        .await
        .map_err(Error::TimetableFetchError)?;

    Ok(timetable
        .inner_timetable
        .into_iter()
        .find(|day| day.date == timetable_date.format(YMD_FORMAT).to_string())
        .ok_or(Error::LessonsForDateNotFound)?
        .time_blocks)
}

pub async fn daily_timetable_usecase(app_repos: &AppRepositories) -> Result<DailyTimetable, Error> {
    let today = Local::now().date_naive();
    //let today = NaiveDate::from_ymd_opt(2026, 04, 28).unwrap();
    let current_time = Local::now().time();

    let when = TimetableWhen::fetch_from_current_timetable(&app_repos, today, current_time).await?;

    let week_start = when.week_start(today)?;

    let timetable_date = match when {
        TimetableWhen::NextWeek => week_start,
        TimetableWhen::Tomorrow => today
            .checked_add_days(Days::new(1))
            .ok_or(Error::TomorrowDateError)?,
        TimetableWhen::Today => today,
    };

    let time_blocks = fetch_timeblocks_of_day(app_repos, week_start, timetable_date).await?;
    let events = super::fetch_events(app_repos, timetable_date)
        .await
        .map_err(Error::EventFetchError)?;

    let daily_time_blocks = time_blocks
        .into_iter()
        .map(|time_block| SubjectEventTimeBlock::merge_from(time_block, &events, timetable_date))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::SubjectEventTimeblockMergeError)?;

    let daily_time_blocks = super::trim_timetable(daily_time_blocks);

    let day_of_week = timetable_date.format("%A").to_string();

    Ok(DailyTimetable {
        day_of_week,
        when,
        time_blocks: daily_time_blocks,
    })
}
