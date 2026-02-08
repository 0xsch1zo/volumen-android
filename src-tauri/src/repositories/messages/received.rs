use std::sync::Arc;

use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable, SingleEntryCache},
    repositories::messages::{
        AttachmentReference, Limit, MessageId, MessagesSource, Page, Receiver,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch recieved messages")]
    ListGetError(#[source] CacheComputeError),
    #[error("failed to fetch recieved message")]
    MessageGetError(#[source] CacheComputeError),
}

#[derive(Clone, Debug)]
pub struct ReceivedMessagePreview {
    pub message_id: MessageId,
    pub sender_name: String,
    pub topic: String,
    pub fragment: String,
    pub send_date: String,
    pub read_date: Option<String>,
    pub has_file_attachment: bool,
}

#[derive(Clone, Debug)]
pub struct ReceivedMessagePreviews {
    pub messages: Vec<ReceivedMessagePreview>,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    pub message_id: MessageId,
    pub sender_name: String,
    pub topic: String,
    pub message: String,
    pub send_date: String,
    pub read_date: Option<String>,
    pub no_reply: bool,
    pub is_archived: bool,
    pub attachments: Vec<AttachmentReference>,
    pub receivers: Vec<Receiver>,
}

impl Keyable<MessageId> for ReceivedMessage {
    fn key(&self) -> MessageId {
        self.message_id
    }
}

#[derive(Debug, Clone)]
pub struct ReceivedMessagesRepository {
    preview_cache: SingleEntryCache<ReceivedMessagePreviews>,
    message_cache: AutoKeyedCache<MessageId, ReceivedMessage>,
}

impl ReceivedMessagesRepository {
    pub fn new() -> Self {
        Self {
            preview_cache: SingleEntryCache::new(),
            message_cache: AutoKeyedCache::new(),
        }
    }

    pub(super) fn delegate<'a>(
        &'a self,
        messages_source: impl MessagesSource + 'a,
    ) -> ReceivedMessagesDelegate<'a> {
        ReceivedMessagesDelegate::new(messages_source, &self.preview_cache, &self.message_cache)
    }
}

pub struct ReceivedMessagesDelegate<'a> {
    messages_source: Arc<dyn MessagesSource + 'a>,
    preview_cache: &'a SingleEntryCache<ReceivedMessagePreviews>,
    message_cache: &'a AutoKeyedCache<MessageId, ReceivedMessage>,
}

impl<'a> ReceivedMessagesDelegate<'a> {
    fn new(
        messages_source: impl MessagesSource + 'a,
        preview_cache: &'a SingleEntryCache<ReceivedMessagePreviews>,
        message_cache: &'a AutoKeyedCache<MessageId, ReceivedMessage>,
    ) -> Self {
        Self {
            messages_source: Arc::new(messages_source),
            preview_cache,
            message_cache,
        }
    }

    pub async fn list(&self, page: Page, limit: Limit) -> Result<ReceivedMessagePreviews, Error> {
        self.preview_cache
            .try_get_with(async {
                self.messages_source
                    .received()
                    .fetch_list(page, limit)
                    .await
            })
            .await
            .map_err(Error::ListGetError)
    }

    pub async fn message(&self, message_id: MessageId) -> Result<ReceivedMessage, Error> {
        self.message_cache
            .try_get_with(&message_id, async {
                self.messages_source
                    .received()
                    .fetch_message(message_id)
                    .await
            })
            .await
            .map_err(Error::MessageGetError)
    }
}
