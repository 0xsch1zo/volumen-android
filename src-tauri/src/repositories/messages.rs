use std::sync::Arc;

use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable, SingleEntryCache},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch recieved messages")]
    RecievedMessagesFetchError(#[source] CacheComputeError),
    #[error("failed to fetch recieved message")]
    RecievedMessageFetchError(#[source] CacheComputeError),
}

pub struct Page(usize);

impl Page {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

pub struct Limit(usize);

impl Limit {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct MessageId(usize);

impl MessageId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct RecievedMessagePreview {
    pub message_id: MessageId,
    pub sender_name: String,
    pub topic: String,
    pub fragment: String,
    pub send_date: String,
    pub read_date: Option<String>,
    pub has_file_attachment: bool,
}

#[derive(Clone, Debug)]
pub struct RecievedMessagePreviews {
    pub messages: Vec<RecievedMessagePreview>,
    pub total: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct AttachmentId(usize);

impl AttachmentId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentReference {
    pub id: AttachmentId,
    pub filename: String,
}

#[derive(Debug, Clone)]
pub struct RecievedMessage {
    pub message_id: MessageId,
    pub sender_name: String,
    pub topic: String,
    pub message: String,
    pub send_date: String,
    pub read_date: Option<String>,
    pub no_reply: bool,
    pub is_archived: bool,
    pub attachments: Vec<AttachmentReference>,
}

impl Keyable<MessageId> for RecievedMessage {
    fn key(&self) -> MessageId {
        self.message_id
    }
}

#[derive(Debug, Clone)]
pub struct MessagesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    recieved_preview_cache: SingleEntryCache<RecievedMessagePreviews>,
    recieved_cache: AutoKeyedCache<MessageId, RecievedMessage>,
}

impl MessagesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            recieved_preview_cache: SingleEntryCache::new(),
            recieved_cache: AutoKeyedCache::new(),
        }
    }

    pub async fn recieved_messages(
        &self,
        page: Page,
        limit: Limit,
    ) -> Result<RecievedMessagePreviews, Error> {
        self.recieved_preview_cache
            .try_get_with(async {
                self.synergia_api
                    .messages()
                    .fetch_recieved_messages(page, limit)
                    .await
            })
            .await
            .map_err(Error::RecievedMessageFetchError)
    }

    pub async fn recieved_message(&self, message_id: MessageId) -> Result<RecievedMessage, Error> {
        self.recieved_cache
            .try_get_with(&message_id, async {
                self.synergia_api
                    .messages()
                    .fetch_recieved_message(message_id)
                    .await
            })
            .await
            .map_err(Error::RecievedMessageFetchError)
    }
}
