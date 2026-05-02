use chrono::{Datelike, Days, Local, NaiveDate, NaiveTime, Weekday};
use itertools::Itertools;
use serde::Serialize;
use thiserror::Error;

use crate::repositories::{
    self,
    calendar::{Month, Year},
    events::Event,
    timetable::{TimeBlocks, WeekStart},
    AppRepositories,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to count to get new date for next week")]
    NextWeekDateError,
    #[error("tomorrow date error")]
    TomorrowDateError,
    #[error("timetable fetch error")]
    TimetableFetchError(#[source] repositories::timetable::Error),
    #[error("failed to parse lesson end time")]
    LessonEndTimeParseError(#[source] chrono::ParseError),
    #[error("lessons for needed timetable date not found")]
    LessonsForDateNotFound,
    #[error("event fetch error")]
    EventFetchError(#[source] repositories::events::Error),
    #[error("failed to fetch calendar")]
    CalendarFetchError(#[source] repositories::calendar::Error),
}

const YMD_FORMAT: &str = "%Y-%m-%d";

#[derive(Serialize, Debug)]
struct TimeBlock {
    start: String,
    end: String,
    subject: String,
    events: Vec<Event>,
}

#[derive(Serialize, Debug)]
enum TimetableWhen {
    Today,
    Tommorow,
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
            (false, None) => Ok(TimetableWhen::Tommorow),
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
                    Ok(TimetableWhen::Tommorow)
                }
            }
            _ => Ok(TimetableWhen::Tommorow),
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
    time_blocks: Vec<Option<TimeBlock>>,
}

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

fn trim_timetable_on_ends(time_blocks: Vec<Option<TimeBlock>>) -> Vec<Option<TimeBlock>> {
    time_blocks
        .into_iter()
        .skip_while(|t| t.is_none())
        .collect_vec()
        .into_iter()
        .rev()
        .skip_while(|t| t.is_none())
        .collect()
}

pub async fn daily_timetable_usecase(app_repos: &AppRepositories) -> Result<DailyTimetable, Error> {
    let today = Local::now().date_naive();
    let current_time = Local::now().time();

    let when = TimetableWhen::fetch_from_current_timetable(&app_repos, today, current_time).await?;

    let week_start = when.week_start(today)?;

    let timetable_date = match when {
        TimetableWhen::NextWeek => week_start,
        TimetableWhen::Tommorow => today
            .checked_add_days(Days::new(1))
            .ok_or(Error::TomorrowDateError)?,
        TimetableWhen::Today => today,
    };

    let time_blocks = fetch_timeblocks_of_day(app_repos, week_start, timetable_date).await?;
    let events = fetch_events(app_repos, timetable_date).await?;

    let daily_time_blocks = time_blocks
        .into_iter()
        .map(|time_block| {
            let events = events
                .iter()
                .filter(|e| {
                    time_block
                        .first()
                        .is_some_and(|t| e.time_from == t.hour_from && e.time_to == t.hour_to)
                })
                .cloned()
                .collect_vec();
            Some(TimeBlock {
                start: time_block.first()?.hour_from.clone(),
                end: time_block.first()?.hour_to.clone(),
                subject: time_block.iter().map(|l| &l.subject.name).join(" | "),
                events,
            })
        })
        .collect_vec();

    let daily_time_blocks = trim_timetable_on_ends(daily_time_blocks);

    let day_of_week = timetable_date.format("%A").to_string();

    Ok(DailyTimetable {
        day_of_week,
        when,
        time_blocks: daily_time_blocks,
    })
}
