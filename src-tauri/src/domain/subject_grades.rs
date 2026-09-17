use std::collections::HashMap;

use itertools::Itertools;
use serde::Serialize;
use thiserror::Error;

use crate::repositories::{
    grades::{self, Grade},
    subjects::{self, Subject, SubjectId},
    AppRepositories,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to fetch grade list")]
    GradeListFetchError(#[source] grades::Error),
    #[error("failed to fetch subjects for list")]
    SubjectListFetchError(#[source] subjects::Error),
}

#[derive(Debug, Serialize)]
pub struct SubjectGrades {
    subject: Subject,
    grades: Vec<Grade>,
}

pub async fn subject_grades_list_usecase(
    app_repos: &AppRepositories,
) -> Result<Vec<SubjectGrades>, Error> {
    let mut subject_grades_map: HashMap<SubjectId, SubjectGrades> = app_repos
        .subjects()
        .list()
        .await
        .map_err(Error::SubjectListFetchError)?
        .into_iter()
        .map(|subject| {
            (
                subject.id,
                SubjectGrades {
                    subject: subject,
                    grades: Vec::new(),
                },
            )
        })
        .collect();

    app_repos
        .grades()
        .list()
        .await
        .map_err(Error::GradeListFetchError)?
        .into_iter()
        .fold(&mut subject_grades_map, |subject_grades_map, grade| {
            subject_grades_map
                .entry(grade.subject.id)
                .and_modify(|subject_grades| subject_grades.grades.push(grade.clone()));
            subject_grades_map
        });

    Ok(subject_grades_map
        .into_values()
        .sorted_by_key(|subject_grade| subject_grade.subject.name.clone())
        .collect_vec())
}
