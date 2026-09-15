use std::collections::HashMap;

use itertools::Itertools;
use thiserror::Error;

use crate::repositories::{
    grades::{self, Grade},
    subjects::{Subject, SubjectId},
    AppRepositories,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to fetch grade list")]
    GradeFetchError(#[source] grades::Error),
}
pub struct SubjectGrades {
    subject: Subject,
    grades: Vec<Grade>,
}

pub async fn list_subject_grades_usecase(
    app_repos: &AppRepositories,
) -> Result<Vec<SubjectGrades>, Error> {
    let mut subject_grades_map: HashMap<SubjectId, Vec<Grade>> = HashMap::new();
    app_repos
        .grades()
        .list()
        .await
        .map_err(Error::GradeFetchError)?
        .into_iter()
        .fold(&mut subject_grades_map, |subject_grades_map, grade| {
            subject_grades_map
                .entry(grade.subject.id)
                .and_modify(|grades| grades.push(grade.clone()))
                .or_insert(vec![grade]);
            subject_grades_map
        });

    Ok(subject_grades_map
        .into_iter()
        .map(|(_, grades)| {
            let subject = grades
            .first()
            .expect(
                "the grade vec should have at least one item if it's part of a subject_grades map",
            )
            .subject
            .clone();
            SubjectGrades { subject, grades }
        })
        .collect_vec())
}
