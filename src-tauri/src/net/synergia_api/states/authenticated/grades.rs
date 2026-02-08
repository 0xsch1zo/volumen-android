use url::Url;

use crate::{
    net::{
        synergia_api::{
            api::grades::{CategoriesResponse, CommentResponse, GradesResponse},
            states::{
                authenticated::{AuthenticatedSynergiaEndpoints, Error},
                AuthenticatedState,
            },
            SYNERGIA_URL,
        },
        SynergiaApi,
    },
    repositories::grades::{Categories, Comment, CommentId, ShallowGrades},
};

#[derive(Debug, Clone)]
pub enum GradesEndpoints {
    Grades,
    Categories,
    Comments(CommentId),
}

impl GradesEndpoints {
    pub fn url(&self) -> Url {
        let endoint = match self {
            GradesEndpoints::Grades => "/gateway/api/2.0/Grades",
            GradesEndpoints::Categories => "/gateway/api/2.0/Grades/Categories",
            GradesEndpoints::Comments(id) => {
                &format!("/gateway/api/2.0/Grades/Comments/{}", id.inner())
            }
        };
        SYNERGIA_URL.join(endoint).unwrap()
    }
}

pub struct GradesManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> GradesManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    pub async fn fetch_self(&self) -> Result<ShallowGrades, Error> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<GradesResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Grades,
            ))
            .await?
            .into())
    }

    pub async fn fetch_categories(&self) -> Result<Categories, Error> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<CategoriesResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Categories,
            ))
            .await?
            .into())
    }

    pub async fn fetch_comment(&self, id: CommentId) -> Result<Comment, Error> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<CommentResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Comments(id),
            ))
            .await?
            .into())
    }
}
