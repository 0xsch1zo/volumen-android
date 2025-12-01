use std::sync::Arc;

use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cache::{Cache, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
    repositories::{
        categories::{self, CategoriesRepository, Category, CategoryId},
        entities::Reference,
        subjects::{self, Subject, SubjectId, SubjectsRepository},
        users::{User, UserId, Users, UsersRepository},
    },
};

pub use comments::{Comment, CommentId};

mod comments;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch grades")]
    GradeFetchFailed(#[source] synergia_api::Error),
    #[error("failed to get subject")]
    FailedToGetSubject(#[source] subjects::Error),
    #[error("failed to get category")]
    FailedToGetCategory(#[source] categories::Error),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct GradeId(usize);

// TODO: move this doghit into synergia_api
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowGrades {
    grades: Vec<ShallowGrade>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowGrade {
    id: GradeId,
    subject: SubjectId,
    category: CategoryId,
    added_by: UserId,
    #[serde(default)]
    comments: Vec<CommentId>,
    grade: String,
    date: String,
    add_date: String,
    semester: usize,
    is_constituent: bool,
    is_semester: bool,
    is_semester_proposition: bool,
    is_final: bool,
    is_final_proposition: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GradeKind {
    Constituent,
    Semester,
    SemesterPropsition,
    Final,
    FinalProposition,
    Unknown,
}

impl GradeKind {
    fn from_shallow_grade(shallow_grade: &ShallowGrade) -> Self {
        // making the dogshit that librus sends digestible to a normal application
        // don't blame me, it is not my fault, I'm just trying to fix this mess
        // NIGHTMARE NIGHTMARE NIGHTMARE
        match shallow_grade {
            &ShallowGrade {
                is_constituent: true,
                is_semester_proposition: false,
                is_semester: false,
                is_final: false,
                is_final_proposition: false,
                ..
            } => Self::Constituent,
            &ShallowGrade {
                is_constituent: false,
                is_semester_proposition: true,
                is_semester: false,
                is_final: false,
                is_final_proposition: false,
                ..
            } => Self::SemesterPropsition,
            &ShallowGrade {
                is_constituent: false,
                is_semester_proposition: false,
                is_semester: true,
                is_final: false,
                is_final_proposition: false,
                ..
            } => Self::Semester,
            &ShallowGrade {
                is_constituent: false,
                is_semester_proposition: false,
                is_semester: false,
                is_final: true,
                is_final_proposition: false,
                ..
            } => Self::Final,
            &ShallowGrade {
                is_constituent: false,
                is_semester_proposition: false,
                is_semester: false,
                is_final: false,
                is_final_proposition: true,
                ..
            } => Self::FinalProposition,
            _ => Self::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Grade {
    id: GradeId,
    added_by: UserId,
    comments: Vec<CommentId>,
    pub subject: Subject,
    pub category: Category,
    pub grade: String,
    pub date: String,
    pub add_date: String,
    pub kind: GradeKind,
}

impl Grade {
    fn from_shallow(shallow: ShallowGrade, subject: Subject, category: Category) -> Self {
        assert_eq!(shallow.subject, subject.id);
        assert_eq!(shallow.category, category.id);
        Grade {
            kind: GradeKind::from_shallow_grade(&shallow),
            id: shallow.id,
            add_date: shallow.add_date,
            added_by: shallow.added_by,
            comments: shallow.comments,
            category,
            subject,
            date: shallow.date,
            grade: shallow.grade,
        }
    }
}

pub struct GradeDetails {
    pub added_by: User,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone)]
pub struct GradesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<Cache>,
    users: UsersRepository,
    subjects: SubjectsRepository,
    categories: CategoriesRepository,
}

impl GradesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>, cache: Arc<Cache>) -> Self {
        let users = UsersRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));
        let subjects = SubjectsRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));
        let categories = CategoriesRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));

        Self {
            synergia_api,
            cache,
            users,
            subjects,
            categories,
        }
    }

    pub async fn grades(&self) -> Result<Vec<Grade>, Error> {
        let shallow_grades = self
            .synergia_api
            .grades()
            .await
            .map_err(|e| Error::GradeFetchFailed(e))?;
        // TODO: figure out an optimal buffering amount
        let grades = stream::iter(shallow_grades.grades)
            .map(async |s| self.assemble_grade(s).await)
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(grades)
    }

    async fn assemble_grade(&self, shallow_grade: ShallowGrade) -> Result<Grade, Error> {
        let subject_fut = self
            .subjects
            .subject(shallow_grade.subject)
            .map_err(|e| Error::FailedToGetSubject(e));

        let category_fut = self
            .categories
            .category(shallow_grade.category)
            .map_err(|e| Error::FailedToGetCategory(e));

        let (subject, category) = tokio::try_join!(subject_fut, category_fut)?;
        Ok(Grade::from_shallow(shallow_grade, subject, category))
    }
}
