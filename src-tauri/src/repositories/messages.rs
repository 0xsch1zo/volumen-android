use std::sync::Arc;

use crate::{
    net::{
        synergia_api::{
            AuthenticatedState, MessagesArchiveManager, MessagesManager, ReceivedMessagesManager,
            SentMessagesManager,
        },
        SynergiaApi,
    },
    repositories::messages::{
        received::{ReceivedMessagesDelegate, ReceivedMessagesRepository},
        sent::{SentMessagesDelegate, SentMessagesRepository},
    },
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

trait MessagesSource: Send + Sync {
    fn sent(&self) -> SentMessagesManager;
    fn received(&self) -> ReceivedMessagesManager;
}

impl MessagesSource for MessagesManager<'_> {
    fn sent(&self) -> SentMessagesManager {
        self.sent()
    }

    fn received(&self) -> ReceivedMessagesManager {
        self.received()
    }
}

impl MessagesSource for MessagesArchiveManager<'_> {
    fn sent(&self) -> SentMessagesManager {
        self.sent()
    }

    fn received(&self) -> ReceivedMessagesManager {
        self.received()
    }
}

#[derive(Debug, Clone)]
pub struct MessagesRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    recieved: ReceivedMessagesRepository,
    sent: SentMessagesRepository,
}

impl MessagesRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            recieved: ReceivedMessagesRepository::new(),
            sent: SentMessagesRepository::new(),
        }
    }

    pub fn received(&self) -> ReceivedMessagesDelegate {
        self.recieved.delegate(self.synergia_api.messages())
    }

    pub fn sent(&self) -> SentMessagesDelegate {
        self.sent.delegate(self.synergia_api.messages())
    }

    pub fn archive(&self) -> ArchiveDelegate {
        ArchiveDelegate::new(self, self.synergia_api.messages())
    }
}

pub struct ArchiveDelegate<'a> {
    message_repository: &'a MessagesRepository,
    messages_manager: MessagesManager<'a>,
}

impl<'a> ArchiveDelegate<'a> {
    pub fn new(
        message_repository: &'a MessagesRepository,
        messages_manager: MessagesManager<'a>,
    ) -> Self {
        Self {
            message_repository,
            messages_manager,
        }
    }

    pub fn received(&'a self) -> ReceivedMessagesDelegate<'a> {
        self.message_repository
            .recieved
            .delegate(self.messages_manager.archive())
    }

    pub fn sent(&self) -> SentMessagesDelegate {
        self.message_repository
            .sent
            .delegate(self.messages_manager.archive())
    }
}
