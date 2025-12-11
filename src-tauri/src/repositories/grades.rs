use std::sync::Arc;

use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
    repositories::{
        grades::{categories::CategoriesRepository, comments::CommentsRepository},
        subjects::{self, Subject, SubjectId, SubjectsRepository},
        users::{self, User, UserId, UsersRepository},
    },
};

pub use categories::{Categories, Category, CategoryId};
pub use comments::{Comment, CommentId};
pub mod categories;
pub mod comments;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch grades")]
    GradeFetchFailed(#[source] CacheComputeError),
    #[error("subject lookup failed")]
    SubjectLookupError(#[source] subjects::Error),
    #[error("category lookup failed")]
    CategoryLookupError(#[source] categories::Error),
    #[error("user lookup failed")]
    UserLookupError(#[source] users::Error),
    #[error("comment lookup failed")]
    CommentLookupError(#[source] comments::Error),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct GradeId(usize);

// TODO: move this dogshit into synergia_api
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowGrades {
    grades: Vec<ShallowGrade>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl Keyable<GradeId> for ShallowGrade {
    fn key(&self) -> GradeId {
        self.id
    }
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

#[derive(Debug)]
pub struct GradeDetails {
    pub added_by: User,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone)]
pub struct GradesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    comments: CommentsRepository,
    users: UsersRepository,
    subjects: SubjectsRepository,
    categories: CategoriesRepository,
    cache: AutoKeyedCache<GradeId, ShallowGrade>,
}

impl GradesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        let comments = CommentsRepository::new(Arc::clone(&synergia_api));
        let users = UsersRepository::new(Arc::clone(&synergia_api));
        let subjects = SubjectsRepository::new(Arc::clone(&synergia_api));
        let categories = CategoriesRepository::new(Arc::clone(&synergia_api));

        Self {
            synergia_api,
            comments,
            users,
            subjects,
            categories,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn grades(&self) -> Result<Vec<Grade>, Error> {
        if self.cache.iter().next().is_none() {
            self.cache
                .try_bulk_insert_with(async {
                    Ok::<_, synergia_api::Error>(self.synergia_api.grades().await?.grades)
                })
                .await
                .map_err(Error::GradeFetchFailed)?;
        }

        // TODO: figure out an optimal buffering amount
        let grades = stream::iter(self.cache.iter().map(|(_, v)| v))
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
            .map_err(Error::SubjectLookupError);

        let category_fut = self
            .categories
            .category(shallow_grade.category)
            .map_err(Error::CategoryLookupError);

        let (subject, category) = tokio::try_join!(subject_fut, category_fut)?;
        Ok(Grade::from_shallow(shallow_grade, subject, category))
    }

    pub async fn details(&self, grade: &Grade) -> Result<GradeDetails, Error> {
        let added_by_fut = self
            .users
            .user(grade.added_by)
            .map_err(Error::UserLookupError);

        let comments_repo = Arc::new(self.comments.clone());
        let comments_fut = stream::iter(grade.comments.iter().cloned())
            .then(|c| {
                // Tauri will shoot you if you won't do otherwise
                // Parallell deosnt' make sense here
                let comments_repo = Arc::clone(&comments_repo);
                async move { comments_repo.comment(c).await }
            })
            .try_collect::<Vec<_>>()
            .map_err(Error::CommentLookupError);
        let (added_by, comments) = tokio::try_join!(added_by_fut, comments_fut)?;
        Ok(GradeDetails { added_by, comments })
    }
}
