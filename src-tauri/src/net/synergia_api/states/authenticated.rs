use std::{any, sync::Arc};

use log::debug;
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use crate::{
    error::{StatefulError, StatefulResultExt},
    net::{
        synergia_api::{
            api::{
                auth::{PortalTokenPair, SynergiaUserId},
                calendar::CalendarResponse,
                me::MeResponse,
                messages::MessageModelConversionError,
                subjects::SubjectsResponse,
                timetable,
                users::UsersResponse,
            },
            authenticated::{
                events::{EventsEndpoints, EventsManager},
                grades::GradesManager,
                messages::MessagesManager,
            },
            authenticators::{MainAuthenticator, MainAuthenticatorError},
            clients::{
                AuthenticatedClientConstructionError, MainAuthenticatedClient, MessagesClient,
                MessagesClientInitError,
            },
            states::authenticated::{grades::GradesEndpoints, messages::MessagesEndpoints},
            SYNERGIA_URL,
        },
        SynergiaApi,
    },
    repositories::{
        calendar::{Calendar, Month, Year},
        me::{ClassId, Me},
        subjects::Subjects,
        timetable::{Timetable, WeekStart},
        users::Users,
    },
    stateful_result,
};

mod events;
mod grades;
pub mod messages;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to initialize api state")]
    StateInitError(#[from] StateInitError),
    #[error("model conversion error")]
    ModelConversionError(#[from] ModelConversionError),
    #[error("failed to send request to endpoint: {endpoint:?}")]
    RequestError {
        endpoint: String,
        #[source]
        source: reqwest_middleware::Error,
    },
    #[error(
        "failed to deserialize response from endpoint: {endpoint:?},\n\twith type: {typename}"
    )]
    ResponseDeserializationError {
        endpoint: String,
        typename: String,
        #[source]
        source: reqwest::Error,
    },
}

#[derive(Error, Debug)]
pub enum StateInitError {
    #[error("main authenticator init error")]
    MainAuthenticatorInitError(#[from] MainAuthenticatorError),
    #[error("main authenticated client construction failure")]
    MainAuthenticatedClientConstructionError(#[from] AuthenticatedClientConstructionError),
    #[error("messages client construction failure")]
    MessagesClientInitError(#[from] MessagesClientInitError),
}

#[derive(Error, Debug)]
pub enum ModelConversionError {
    // TODO: this should probably be a better error
    #[error("message model conversion error")]
    MessageConvError(#[from] MessageModelConversionError),
    #[error("timetable model conversion error")]
    TimetableConvError(#[source] timetable::ModelConversionError),
}

#[derive(Debug)]
pub struct AuthenticatedState {
    main_client: MainAuthenticatedClient,
    messages_client: MessagesClient, // we use a different client because auth works differently
    main_authenticator: Arc<MainAuthenticator>,
}

impl AuthenticatedState {
    async fn init(
        user_id: SynergiaUserId,
        portal_creds: PortalTokenPair,
    ) -> Result<Self, StatefulError<(SynergiaUserId, PortalTokenPair), StateInitError>> {
        let main_authenticator = stateful_result! { (user_id, portal_creds) =>
            MainAuthenticator::init(user_id, portal_creds.clone())
                    .await
                    .map(Arc::new)
                    .map_err(StateInitError::MainAuthenticatorInitError)
        };

        let main_client = stateful_result! { (user_id, portal_creds) =>
            MainAuthenticatedClient::try_new(Arc::clone(&main_authenticator))
                .map_err(StateInitError::MainAuthenticatedClientConstructionError)
        };

        let messages_client = stateful_result! { (user_id, portal_creds) =>
            MessagesClient::init(Arc::clone(&main_authenticator))
                .await
                .map_err(StateInitError::MessagesClientInitError)
        };

        Ok(Self {
            main_client,
            messages_client,
            main_authenticator,
        })
    }
}

#[derive(Debug, Clone)]
enum AuthenticatedSynergiaEndpoints {
    Me,
    Users,
    Subjects,
    Grades(GradesEndpoints),
    Messages(MessagesEndpoints),
    Timetable {
        week_start: String,
    },
    Calendar {
        class_id: usize,
        year: i32,
        month: u8,
    },
    Events(EventsEndpoints),
}

impl AuthenticatedSynergiaEndpoints {
    fn url(&self) -> Url {
        match self {
            AuthenticatedSynergiaEndpoints::Me => SYNERGIA_URL.join("/gateway/api/2.0/Me").unwrap(),
            AuthenticatedSynergiaEndpoints::Users => {
                SYNERGIA_URL.join("/gateway/api/2.0/Users").unwrap()
            }
            AuthenticatedSynergiaEndpoints::Subjects => {
                SYNERGIA_URL.join("/gateway/api/2.0/Subjects").unwrap()
            }
            AuthenticatedSynergiaEndpoints::Timetable { week_start } => SYNERGIA_URL
                .join(&format!(
                    "/gateway/api/2.0/Timetables?weekStart={week_start}"
                ))
                .unwrap(),
            AuthenticatedSynergiaEndpoints::Calendar {
                class_id,
                year,
                month,
            } => SYNERGIA_URL
                .join(&format!(
                    "/gateway/api/2.0/Calendars/{}?year={}&month={}",
                    class_id, year, month
                ))
                .unwrap(),
            AuthenticatedSynergiaEndpoints::Grades(grades) => grades.url(),
            AuthenticatedSynergiaEndpoints::Messages(messages) => messages.url(),
            AuthenticatedSynergiaEndpoints::Events(events) => events.url(),
        }
    }
}

// We're using the api of the new ui
impl SynergiaApi<AuthenticatedState> {
    pub async fn init(
        user_id: SynergiaUserId,
        portal_creds: PortalTokenPair,
    ) -> Result<Self, StatefulError<(SynergiaUserId, PortalTokenPair), Error>> {
        Ok(Self {
            state: AuthenticatedState::init(user_id, portal_creds)
                .await
                .map_stateful_err(Error::StateInitError)?,
        })
    }

    async fn fetch_synergia_endpoint<T: DeserializeOwned>(
        &self,
        endpoint: AuthenticatedSynergiaEndpoints,
    ) -> Result<T, Error> {
        debug!("fetching {endpoint:?}");
        let resource = self
            .state
            .main_client
            .as_inner()
            .get(endpoint.url())
            .send()
            .await
            .map_err(|e| Error::RequestError {
                endpoint: format!("{endpoint:?}"),
                source: e,
            })?
            .json::<T>()
            .await
            .map_err(|e| Error::ResponseDeserializationError {
                endpoint: format!("{endpoint:?}"),
                typename: any::type_name::<T>().to_owned(),
                source: e,
            })?;
        debug!("fetched {endpoint:?} succesfully");
        Ok(resource)
    }

    pub async fn fetch_me(&self) -> Result<Me, Error> {
        Ok(self
            .fetch_synergia_endpoint::<MeResponse>(AuthenticatedSynergiaEndpoints::Me)
            .await?
            .me
            .into())
    }

    pub async fn fetch_users(&self) -> Result<Users, Error> {
        Ok(self
            .fetch_synergia_endpoint::<UsersResponse>(AuthenticatedSynergiaEndpoints::Users)
            .await?
            .into())
    }

    pub async fn fetch_subjects(&self) -> Result<Subjects, Error> {
        Ok(self
            .fetch_synergia_endpoint::<SubjectsResponse>(AuthenticatedSynergiaEndpoints::Subjects)
            .await?
            .into())
    }

    pub async fn fetch_timetable(&self, week_start: WeekStart) -> Result<Timetable, Error> {
        Ok(self
            .fetch_synergia_endpoint::<timetable::Timetable>(
                AuthenticatedSynergiaEndpoints::Timetable {
                    week_start: week_start.into_inner(),
                },
            )
            .await?
            .try_into()
            .map_err(|e| {
                Error::ModelConversionError(ModelConversionError::TimetableConvError(e))
            })?)
    }

    pub async fn fetch_calendar(
        &self,
        class_id: ClassId,
        year: Year,
        month: Month,
    ) -> Result<Calendar, Error> {
        Ok(self
            .fetch_synergia_endpoint::<CalendarResponse>(AuthenticatedSynergiaEndpoints::Calendar {
                class_id: class_id.as_inner(),
                year: year.into_inner(),
                month: month.into_inner(),
            })
            .await?
            .calendar
            .into())
    }

    pub fn grades(&self) -> GradesManager<'_> {
        GradesManager::new(&self)
    }

    pub fn messages(&self) -> MessagesManager<'_> {
        MessagesManager::new(&self)
    }

    pub fn events(&self) -> EventsManager<'_> {
        EventsManager::new(&self)
    }
}
