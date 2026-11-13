use url::Url;

use crate::{
    net::{
        synergia_api::{
            api::subjects::{SubjectResponse, SubjectsResponse},
            authenticated::AuthenticatedSynergiaEndpoints,
            AuthenticatedState, AuthenticatedSynergiaApiError, SYNERGIA_URL,
        },
        SynergiaApi,
    },
    repositories::subjects::{Subject, SubjectId},
};

#[derive(Debug, Clone)]
pub enum SubjectsEndpoints {
    Subjects,
    Subject(usize),
}

impl SubjectsEndpoints {
    pub fn url(&self) -> Url {
        let endpoint = match self {
            SubjectsEndpoints::Subjects => "/gateway/api/2.0/Subjects",
            SubjectsEndpoints::Subject(id) => &format!("/gateway/api/2.0/Subjects/{id}"),
        };
        SYNERGIA_URL.join(endpoint).unwrap()
    }
}

pub struct SubjectsManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> SubjectsManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    pub async fn fetch_list(&self) -> Result<Vec<Subject>, AuthenticatedSynergiaApiError> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<SubjectsResponse>(AuthenticatedSynergiaEndpoints::Subjects(
                SubjectsEndpoints::Subjects,
            ))
            .await?
            .into())
    }

    pub async fn fetch_subject(
        &self,
        id: SubjectId,
    ) -> Result<Subject, AuthenticatedSynergiaApiError> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<SubjectResponse>(AuthenticatedSynergiaEndpoints::Subjects(
                SubjectsEndpoints::Subject(id.into_inner()),
            ))
            .await?
            .into())
    }
}
