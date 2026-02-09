use std::{collections::HashMap, num::ParseIntError};

use serde::Deserialize;
use thiserror::Error;

use crate::repositories::timetable as models;

// Because of the overall stupidy with which this endpoint behaves I need to reimplement most of
// the types used here and cannot use those defined in the rest of the api. Again, thanks librus <3

#[derive(Error, Debug)]
pub enum ModelConversionError {
    #[error("failed to parse lesson number as number")]
    LessonNumberParseError(#[source] ParseIntError),
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Subject {
    name: String,
    short: String,
}

impl From<Subject> for models::Subject {
    fn from(value: Subject) -> Self {
        Self {
            name: value.name,
            short: value.short,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct User {
    first_name: String,
    last_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Lesson {
    #[serde(rename = "LessonNo")]
    number: String,
    subject: Subject,
    teacher: User,
    hour_from: String,
    hour_to: String,
    is_canceled: bool,
    #[serde(rename = "IsSubstitutionClass")]
    is_substitution: bool,
}

impl TryFrom<Lesson> for models::Lesson {
    type Error = ModelConversionError;

    fn try_from(value: Lesson) -> Result<Self, Self::Error> {
        Ok(Self {
            number: value
                .number
                .parse()
                .map_err(ModelConversionError::LessonNumberParseError)?,
            subject: value.subject.into(),
            teacher_name: format!("{} {}", value.teacher.first_name, value.teacher.last_name),
            hour_from: value.hour_from,
            hour_to: value.hour_to,
            is_canceled: value.is_canceled,
            is_substitution: value.is_substitution,
        })
    }
}

type TimeBlock = Vec<Lesson>;

type TimeBlocks = Vec<TimeBlock>;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Timetable {
    timetable: HashMap<String, TimeBlocks>,
}

impl TryFrom<Timetable> for models::Timetable {
    type Error = ModelConversionError;

    fn try_from(value: Timetable) -> Result<Self, Self::Error> {
        Ok(Self {
            timetable: value
                .timetable
                .into_iter()
                // yeah I know it looks like shit..., but I had to, sorry
                .map(|(date, time_blocks)| {
                    let time_blocks = time_blocks
                        .into_iter()
                        .map(|time_block| {
                            time_block
                                .into_iter()
                                .map(|lesson| lesson.try_into())
                                .collect::<Result<models::TimeBlock, _>>()
                        })
                        .collect::<Result<models::TimeBlocks, _>>()?;
                    Ok(models::Day {
                        date: models::Date::new(date),
                        time_blocks,
                    })
                })
                .collect::<Result<_, _>>()?,
        })
    }
}
