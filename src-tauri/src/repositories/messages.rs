use std::sync::Arc;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::messages::{received::ReceivedMessageRepository, sent::SentMessageRepository},
};

pub mod received;
pub mod sent;

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
pub struct ReceiverId(usize);

impl ReceiverId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Clone)]
pub struct Receiver {
    pub receiver_id: ReceiverId,
    pub name: String,
    pub read_date: String,
}

#[derive(Debug, Clone)]
pub struct MessagesRepository {
    recieved: ReceivedMessageRepository,
    sent: SentMessageRepository,
}

impl MessagesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            recieved: ReceivedMessageRepository::new(Arc::clone(&synergia_api)),
            sent: SentMessageRepository::new(Arc::clone(&synergia_api)),
        }
    }

    pub fn received(&self) -> &ReceivedMessageRepository {
        &self.recieved
    }

    pub fn sent(&self) -> &SentMessageRepository {
        &self.sent
    }
}
