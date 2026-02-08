use std::sync::Arc;

use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{CacheComputeError, Keyable, SingleEntryCache},
    net::{
        synergia_api::{AuthenticatedState, AuthenticatedSynergiaApiError},
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

#[derive(Serialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[serde(transparent)]
pub struct GradeId(usize);

impl GradeId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ShallowGrade {
    pub id: GradeId,
    pub subject: SubjectId,
    pub category: CategoryId,
    pub added_by: UserId,
    pub comments: Vec<CommentId>,
    pub grade: String,
    pub date: String,
    pub add_date: String,
    pub semester: usize,
    pub is_constituent: bool,
    pub is_semester: bool,
    pub is_semester_proposition: bool,
    pub is_final: bool,
    pub is_final_proposition: bool,
}

impl Keyable<GradeId> for ShallowGrade {
    fn key(&self) -> GradeId {
        self.id
    }
}

pub type ShallowGrades = Vec<ShallowGrade>;

#[derive(Serialize, Debug)]
pub enum GradeKind {
    Constituent,
    Semester,
    SemesterPropsition,
    Final,
    FinalProposition,
    Unknown,
}

impl GradeKind {
    // TODO: maybe allow for multiple types of grade at once
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

#[derive(Serialize, Debug)]
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

type Grades = Vec<Grade>;

#[derive(Debug)]
pub struct GradeDetails {
    pub added_by: User,
    pub comments: Vec<Comment>,
}
struct GradesFactory<'a> {
    subject_repo: &'a SubjectsRepository,
    categories_repo: &'a CategoriesRepository,
}

impl<'a> GradesFactory<'a> {
    fn new(
        subject_repo: &'a SubjectsRepository,
        categories_repo: &'a CategoriesRepository,
    ) -> Self {
        Self {
            subject_repo,
            categories_repo,
        }
    }

    async fn create_from_shallow(&self, shallow: ShallowGrade) -> Result<Grade, Error> {
        let subject_fut = self
            .subject_repo
            .subject(shallow.subject)
            .map_err(Error::SubjectLookupError);

        let category_fut = self
            .categories_repo
            .category(shallow.category)
            .map_err(Error::CategoryLookupError);

        let (subject, category) = tokio::try_join!(subject_fut, category_fut)?;
        Ok(Grade {
            kind: GradeKind::from_shallow_grade(&shallow),
            id: shallow.id,
            add_date: shallow.add_date,
            added_by: shallow.added_by,
            comments: shallow.comments,
            category,
            subject,
            date: shallow.date,
            grade: shallow.grade,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GradesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    comments: CommentsRepository,
    users: UsersRepository,
    subjects: SubjectsRepository,
    categories: CategoriesRepository,
    cache: SingleEntryCache<ShallowGrades>,
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
            cache: SingleEntryCache::new(),
        }
    }

    pub async fn list(&self) -> Result<Grades, Error> {
        let shallow_grades = self
            .cache
            .try_get_with(async {
                Ok::<_, AuthenticatedSynergiaApiError>(
                    self.synergia_api.grades().fetch_self().await?,
                )
            })
            .await
            .map_err(Error::GradeFetchFailed)?;

        let grades_factory = GradesFactory::new(&self.subjects, &self.categories);
        // TODO: figure out an optimal buffering amount
        let grades = stream::iter(shallow_grades)
            .map(async |s| grades_factory.create_from_shallow(s).await)
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(grades)
    }

    pub async fn details(&self, grade: &Grade) -> Result<GradeDetails, Error> {
        let added_by_fut = self
            .users
            .user(grade.added_by)
            .map_err(Error::UserLookupError);

        let comments_repo = Arc::new(self.comments.clone());
        let comments_fut = stream::iter(grade.comments.iter().cloned())
            .then(|c| {
                // Tauri will shoot you if you do otherwise
                // Parallel isn't worth it here
                let comments_repo = Arc::clone(&comments_repo);
                async move { comments_repo.comment(c).await }
            })
            .try_collect::<Vec<_>>()
            .map_err(Error::CommentLookupError);
        let (added_by, comments) = tokio::try_join!(added_by_fut, comments_fut)?;
        Ok(GradeDetails { added_by, comments })
    }
}
