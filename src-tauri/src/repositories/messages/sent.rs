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
    #[error("failed to get message list")]
    ListGetError(#[source] CacheComputeError),
    #[error("failed to get message ")]
    MessageGetError(#[source] CacheComputeError),
}

#[derive(Debug, Clone)]
pub struct SentMessagePreview {
    pub message_id: MessageId,
    pub receiver_name: String,
    pub topic: String,
    pub fragment: String,
    pub send_date: String,
    pub has_file_attachment: bool,
}

#[derive(Debug, Clone)]
pub struct SentMessagePreviews {
    pub messages: Vec<SentMessagePreview>,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct SentMessage {
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

impl Keyable<MessageId> for SentMessage {
    fn key(&self) -> MessageId {
        self.message_id
    }
}

#[derive(Debug, Clone)]
pub struct SentMessagesRepository {
    preview_cache: SingleEntryCache<SentMessagePreviews>,
    message_cache: AutoKeyedCache<MessageId, SentMessage>,
}

impl SentMessagesRepository {
    pub fn new() -> Self {
        Self {
            preview_cache: SingleEntryCache::new(),
            message_cache: AutoKeyedCache::new(),
        }
    }

    pub(super) fn delegate<'a>(
        &'a self,
        messages_source: impl MessagesSource + 'a,
    ) -> SentMessagesDelegate<'a> {
        SentMessagesDelegate::new(messages_source, &self.preview_cache, &self.message_cache)
    }
}

pub struct SentMessagesDelegate<'a> {
    messages_source: Arc<dyn MessagesSource + 'a>,
    preview_cache: &'a SingleEntryCache<SentMessagePreviews>,
    message_cache: &'a AutoKeyedCache<MessageId, SentMessage>,
}

impl<'a> SentMessagesDelegate<'a> {
    fn new(
        messages_source: impl MessagesSource + 'a,
        preview_cache: &'a SingleEntryCache<SentMessagePreviews>,
        message_cache: &'a AutoKeyedCache<MessageId, SentMessage>,
    ) -> Self {
        Self {
            messages_source: Arc::new(messages_source),
            preview_cache,
            message_cache,
        }
    }

    pub async fn list(&self, page: Page, limit: Limit) -> Result<SentMessagePreviews, Error> {
        self.preview_cache
            .try_get_with(async { self.messages_source.sent().fetch_list(page, limit).await })
            .await
            .map_err(Error::ListGetError)
    }

    pub async fn message(&self, message_id: MessageId) -> Result<SentMessage, Error> {
        self.message_cache
            .try_get_with(&message_id, async {
                self.messages_source.sent().fetch_message(message_id).await
            })
            .await
            .map_err(Error::MessageGetError)
    }
}
