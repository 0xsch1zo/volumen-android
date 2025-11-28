use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    cache::{Cache, Keyable},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        categories::Category,
        entities::Reference,
        subjects::Subject,
        users::{User, UserId},
        Result,
    },
};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub struct CommentId(usize);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Comment {
    id: CommentId,
    text: String,
}

impl From<Reference> for CommentId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

impl Keyable<CommentId> for Comment {
    fn key(&self) -> CommentId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(from = "Reference")]
pub struct GradeId(usize);

impl From<Reference> for GradeId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Grade {
    id: GradeId,
    subject: Subject,
    category: Category,
    added_by: User,
    comments: Vec<Comment>,
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

#[derive(Debug)]
pub struct GradesRepo {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<Cache>,
}

impl GradesRepo {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>, cache: Arc<Cache>) -> Self {
        Self {
            synergia_api,
            cache,
        }
    }

    pub async fn grades(&self) -> Result<()> {
        let shallow_grades = self.synergia_api.grades().await?;
        Ok(())
    }
}
