use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SynergiaUserId(usize);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SynergiaAccount {
    pub id: SynergiaUserId,
    pub group: String,
    pub login: String,
    pub student_name: String,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SynergiaAccounts {
    #[serde(rename = "accounts")]
    pub inner: Vec<SynergiaAccount>,
}

// Generic reference used for internal purposes
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Reference {
    id: usize,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LessonId(usize);

impl From<Reference> for LessonId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectId(usize);

impl From<Reference> for SubjectId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StudentId(usize);

impl From<Reference> for StudentId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CategoryId(usize);

impl From<Reference> for CategoryId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserId(usize);

impl From<Reference> for UserId {
    fn from(value: Reference) -> Self {
        Self(value.id)
    }
}
