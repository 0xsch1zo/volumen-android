use std::{any, cell::LazyCell};

use log::debug;
use serde::de::DeserializeOwned;
use url::Url;

use crate::{
    net::{
        synergia_api::{
            api::messages::{received, sent},
            states::{
                authenticated::{AuthenticatedSynergiaEndpoints, Error, ModelConversionError},
                AuthenticatedState,
            },
            MESSAGES_URL, SYNERGIA_URL,
        },
        SynergiaApi,
    },
    repositories::messages::{
        received::{ReceivedMessage, ReceivedMessagePreviews},
        sent::{SentMessage, SentMessagePreviews},
        Limit, MessageId, Page,
    },
};

pub const MESSAGES_AUTHORIZATION_URL: LazyCell<Url> =
    LazyCell::new(|| MessagesEndpoints::Authorization.url());

#[derive(Debug, Clone)]
pub(super) enum MessagesEndpoints {
    Authorization,
    CurrentMessages(MessagesSourceEndpoints),
    ArchivedMessages(MessagesSourceEndpoints),
}

impl MessagesEndpoints {
    pub fn url(&self) -> Url {
        match self {
            MessagesEndpoints::Authorization => SYNERGIA_URL.join("/wiadomosci3").unwrap(),
            MessagesEndpoints::CurrentMessages(endpoints) => endpoints.current_url(),
            MessagesEndpoints::ArchivedMessages(endpoints) => endpoints.archive_url(),
        }
    }
}

#[derive(Debug, Clone)]
enum MessagesSourceEndpoints {
    ReceivedMessage { id: usize },
    ReceivedMessages { page: usize, limit: usize },
    SentMessage { id: usize },
    SentMessages { page: usize, limit: usize },
}

impl MessagesSourceEndpoints {
    fn current_url(&self) -> Url {
        let endpoint = match self {
            MessagesSourceEndpoints::ReceivedMessage { id } => &format!("/api/inbox/messages/{id}"),
            MessagesSourceEndpoints::ReceivedMessages { page, limit } => {
                &format!("/api/inbox/messages?page={page}&limit={limit}")
            }
            MessagesSourceEndpoints::SentMessage { id } => &format!("/api/outbox/messages/{id}"),
            MessagesSourceEndpoints::SentMessages { page, limit } => {
                &format!("/api/outbox/messages?page={page}&limit={limit}")
            }
        };
        MESSAGES_URL.join(endpoint).unwrap()
    }

    fn archive_url(&self) -> Url {
        let endpoint = match self {
            MessagesSourceEndpoints::ReceivedMessage { id } => {
                &format!("/api/archive/inbox/messages/{id}")
            }
            MessagesSourceEndpoints::ReceivedMessages { page, limit } => {
                &format!("/api/archive/inbox/messages?page={page}&limit={limit}")
            }
            MessagesSourceEndpoints::SentMessage { id } => {
                &format!("/api/archive/outbox/messages/{id}")
            }
            MessagesSourceEndpoints::SentMessages { page, limit } => {
                &format!("/api/archive/outbox/messages?page={page}&limit={limit}")
            }
        };
        MESSAGES_URL.join(endpoint).unwrap()
    }
}

enum MessagesSource {
    Current,
    Archive,
}

impl MessagesSource {
    fn into_messages_endpoint(
        &self,
        source_endpoint: MessagesSourceEndpoints,
    ) -> MessagesEndpoints {
        match self {
            Self::Current => MessagesEndpoints::CurrentMessages(source_endpoint),
            Self::Archive => MessagesEndpoints::ArchivedMessages(source_endpoint),
        }
    }
}

pub struct MessagesManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> MessagesManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    async fn fetch_message_endpoint<T: DeserializeOwned>(
        &self,
        endpoint: AuthenticatedSynergiaEndpoints,
    ) -> Result<T, Error> {
        debug!("fetching message endpoint: {endpoint:?}");
        let resource = self
            .synergia_api
            .state
            .messages_client
            .as_inner()
            .get(endpoint.url())
            .send()
            .await
            .map_err(|e| Error::RequestError {
                endpoint: endpoint.clone(),
                source: e,
            })?
            .json::<T>()
            .await
            .map_err(|e| Error::ResponseDeserializationError {
                endpoint: endpoint.clone(),
                typename: any::type_name::<T>().to_owned(),
                source: e,
            })?;
        debug!("fetched message endpoint: {endpoint:?} succesfully");
        Ok(resource)
    }

    pub fn received(&self) -> ReceivedMessagesManager {
        ReceivedMessagesManager::new(self, MessagesSource::Current)
    }

    pub fn sent(&self) -> SentMessagesManager {
        SentMessagesManager::new(self, MessagesSource::Current)
    }

    pub fn archive(&self) -> MessagesArchiveManager {
        MessagesArchiveManager::new(self)
    }
}

pub struct ReceivedMessagesManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
    source: MessagesSource,
}

impl<'a> ReceivedMessagesManager<'a> {
    fn new(messages_manager: &'a MessagesManager<'a>, source: MessagesSource) -> Self {
        Self {
            messages_manager,
            source,
        }
    }

    pub async fn fetch_list(
        &self,
        page: Page,
        limit: Limit,
    ) -> Result<ReceivedMessagePreviews, Error> {
        let source_endopint = MessagesSourceEndpoints::ReceivedMessages {
            page: page.into_inner(),
            limit: limit.into_inner(),
        };

        Ok(self
            .messages_manager
            .fetch_message_endpoint::<received::ReceivedMessagePreviews>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endopint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }

    pub async fn fetch_message(&self, message_id: MessageId) -> Result<ReceivedMessage, Error> {
        let source_endpoint = MessagesSourceEndpoints::ReceivedMessage {
            id: message_id.into_inner(),
        };

        Ok(self
            .messages_manager
            .fetch_message_endpoint::<received::ReceivedMessageResponse>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }
}

pub struct SentMessagesManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
    source: MessagesSource,
}

impl<'a> SentMessagesManager<'a> {
    fn new(messages_manager: &'a MessagesManager, source: MessagesSource) -> Self {
        Self {
            messages_manager,
            source,
        }
    }

    pub async fn fetch_list(&self, page: Page, limit: Limit) -> Result<SentMessagePreviews, Error> {
        let source_endpoint = MessagesSourceEndpoints::SentMessages {
            page: page.into_inner(),
            limit: limit.into_inner(),
        };
        Ok(self
            .messages_manager
            .fetch_message_endpoint::<sent::SentMessagePreviews>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }

    pub async fn fetch_message(&self, message_id: MessageId) -> Result<SentMessage, Error> {
        let source_endpoint = MessagesSourceEndpoints::SentMessage {
            id: message_id.into_inner(),
        };
        Ok(self
            .messages_manager
            .fetch_message_endpoint::<sent::SentMessageResponse>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }
}

pub struct MessagesArchiveManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
}

impl<'a> MessagesArchiveManager<'a> {
    pub fn new(messages_manager: &'a MessagesManager<'a>) -> Self {
        Self { messages_manager }
    }

    pub fn received(&self) -> ReceivedMessagesManager {
        ReceivedMessagesManager::new(&self.messages_manager, MessagesSource::Archive)
    }

    pub fn sent(&self) -> SentMessagesManager {
        SentMessagesManager::new(&self.messages_manager, MessagesSource::Archive)
    }
}
