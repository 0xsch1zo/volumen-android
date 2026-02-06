use std::sync::Arc;

use thiserror::Error;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        grades::{self, Grade, GradeDetails, GradesRepository},
        messages::{
            self, Limit, MessageId, MessagesRepository, Page, RecievedMessage,
            RecievedMessagePreviews,
        },
        Result,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to get grades")]
    GradesFetchError(#[source] grades::Error),
    #[error("failed to get grade details")]
    GradeDetailsFetchError(#[source] grades::Error),
    #[error("failed to get messages")]
    MessagesFetchError(#[source] messages::Error),
}

#[derive(Debug)]
pub struct MainRepository {
    grades: GradesRepository,
    messages: MessagesRepository,
}

impl MainRepository {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        let synergia_api = Arc::new(synergia_api);

        let grades = GradesRepository::new(Arc::clone(&synergia_api));
        let messages = MessagesRepository::new(Arc::clone(&synergia_api));

        Self { grades, messages }
    }

    #[allow(unused)]
    pub async fn grades(&self) -> Result<Vec<Grade>> {
        Ok(self
            .grades
            .grades()
            .await
            .map_err(Error::GradesFetchError)?)
    }

    #[allow(unused)]
    pub async fn grade_details(&self, grade: &Grade) -> Result<GradeDetails> {
        Ok(self
            .grades
            .details(&grade)
            .await
            .map_err(Error::GradeDetailsFetchError)?)
    }

    // TODO: we may need more modularization when it comes to messages repo
    #[allow(unused)]
    pub async fn recieved_messages(
        &self,
        page: Page,
        limit: Limit,
    ) -> Result<RecievedMessagePreviews> {
        Ok(self
            .messages
            .recieved_messages(page, limit)
            .await
            .map_err(Error::MessagesFetchError)?)
    }

    #[allow(unused)]
    pub async fn recieved_message(&self, message_id: MessageId) -> Result<RecievedMessage> {
        Ok(self
            .messages
            .recieved_message(message_id)
            .await
            .map_err(Error::MessagesFetchError)?)
    }
}
