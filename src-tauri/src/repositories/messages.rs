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
    RecievedMessageFetchError(#[from] CacheComputeError),
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

#[derive(Clone, Debug)]
pub struct MessageId(usize);

impl MessageId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
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

pub struct AttachmentId(usize);

struct AttachmentReference {
    id: AttachmentId,
    filename: String,
}

pub struct RecievedMessage {
    message_id: MessageId,
    sender_name: String,
    topic: String,
    message: String,
    send_date: String,
    read_date: Option<String>,
    no_reply: bool,
    is_archived: bool,
    attachments: Vec<AttachmentReference>,
}

#[derive(Debug, Clone)]
pub struct MessagesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    recieved_preview_cache: SingleEntryCache<RecievedMessagePreviews>,
}

impl MessagesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            recieved_preview_cache: SingleEntryCache::new(),
        }
    }

    pub async fn recieved(
        &self,
        page: Page,
        limit: Limit,
    ) -> Result<RecievedMessagePreviews, Error> {
        self.recieved_preview_cache
            .try_get_with(async {
                self.synergia_api
                    .messages()
                    .fetch_recieved(page, limit)
                    .await
            })
            .await
            .map_err(Error::RecievedMessageFetchError)
    }
}
