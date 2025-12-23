use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    net::synergia_api::api::{subjects::SubjectId, users::UserId, Reference},
    repositories::grades as models,
};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub(super) struct CommentId(usize);

impl From<Reference> for CommentId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<CommentId> for models::CommentId {
    fn from(value: CommentId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Comment {
    id: CommentId,
    text: String,
}

impl From<Comment> for models::Comment {
    fn from(value: Comment) -> Self {
        Self {
            id: value.id.into(),
            text: value.text,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CommentResponse {
    comment: Comment,
}

impl From<CommentResponse> for models::Comment {
    fn from(value: CommentResponse) -> Self {
        value.comment.into()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub(super) struct CategoryId(usize);

impl From<Reference> for CategoryId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<CategoryId> for models::CategoryId {
    fn from(value: CategoryId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Category {
    id: CategoryId,
    name: String,
    count_to_the_average: bool,
    weight: Option<usize>,
}

impl From<Category> for models::Category {
    fn from(value: Category) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
            count_to_the_average: value.count_to_the_average,
            weight: value.weight,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CategoriesResponse {
    categories: Vec<Category>,
}

impl From<CategoriesResponse> for models::Categories {
    fn from(value: CategoriesResponse) -> Self {
        value.categories.into_iter().map(Into::into).collect_vec()
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[serde(from = "Reference")]
pub(super) struct GradeId(usize);

impl From<Reference> for GradeId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<GradeId> for models::GradeId {
    fn from(value: GradeId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub(super) struct ShallowGrade {
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

impl From<ShallowGrade> for models::ShallowGrade {
    fn from(value: ShallowGrade) -> Self {
        Self {
            id: value.id.into(),
            subject: value.subject.into(),
            category: value.category.into(),
            added_by: value.added_by.into(),
            comments: value.comments.into_iter().map(Into::into).collect_vec(),
            grade: value.grade,
            date: value.date,
            add_date: value.add_date,
            semester: value.semester,
            is_constituent: value.is_constituent,
            is_semester: value.is_semester,
            is_semester_proposition: value.is_semester_proposition,
            is_final: value.is_final,
            is_final_proposition: value.is_final_proposition,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GradesResponse {
    grades: Vec<ShallowGrade>,
}

impl From<GradesResponse> for models::ShallowGrades {
    fn from(value: GradesResponse) -> Self {
        value.grades.into_iter().map(Into::into).collect_vec()
    }
}
