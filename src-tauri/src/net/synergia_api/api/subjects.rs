use serde::{Deserialize, Serialize};

use crate::{net::synergia_api::api::Reference, repositories::subjects as models};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub(super) struct SubjectId(usize);

impl From<Reference> for SubjectId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<SubjectId> for models::SubjectId {
    fn from(value: SubjectId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Subject {
    numeric_identifier: SubjectId,
    name: String,
    extracurricular: bool,
}

impl From<Subject> for models::Subject {
    fn from(value: Subject) -> Self {
        Self {
            id: value.numeric_identifier.into(),
            name: value.name,
            is_extracurricular: value.extracurricular,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubjectsResponse {
    data: Vec<Subject>,
}

impl From<SubjectsResponse> for models::Subjects {
    fn from(value: SubjectsResponse) -> Self {
        value.data.into_iter().map(Into::into).collect()
    }
}
