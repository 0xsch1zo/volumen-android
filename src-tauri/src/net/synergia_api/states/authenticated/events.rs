use url::Url;

use crate::{
    net::{
        synergia_api::{
            api::events::EventsResponse, authenticated::AuthenticatedSynergiaEndpoints,
            AuthenticatedState, AuthenticatedSynergiaApiError, SYNERGIA_URL,
        },
        SynergiaApi,
    },
    repositories::events::ShallowEvent,
};

#[derive(Debug, Clone)]
pub enum EventsEndpoints {
    Events,
    Categories,
}

impl EventsEndpoints {
    pub fn url(&self) -> Url {
        let endpoint = match self {
            EventsEndpoints::Events => "/gateway/api/2.0/HomeWorks", // for fucks
            EventsEndpoints::Categories => "/gateway/2.0/HomeWorks/Categories",
        };
        SYNERGIA_URL.join(endpoint).unwrap()
    }
}

pub struct EventsManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> EventsManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    pub async fn list(&self) -> Result<Vec<ShallowEvent>, AuthenticatedSynergiaApiError> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<EventsResponse>(AuthenticatedSynergiaEndpoints::Events(
                EventsEndpoints::Events,
            ))
            .await?
            .events
            .into_iter()
            .map(Into::into)
            .collect())
    }
}
