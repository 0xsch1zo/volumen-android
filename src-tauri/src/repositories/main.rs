use std::sync::Arc;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        grades::{Grade, GradeDetails, GradesRepository},
        messages::{Limit, Page, RecievedMessagePreviews},
        Result,
    },
};

#[derive(Debug)]
pub struct MainRepository {
    grades: GradesRepository,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
}

impl MainRepository {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        let synergia_api = Arc::new(synergia_api);

        let grades = GradesRepository::new(Arc::clone(&synergia_api));

        Self {
            grades,
            synergia_api,
        }
    }

    pub async fn grades(&self) -> Result<Vec<Grade>> {
        Ok(self.grades.grades().await?)
    }

    pub async fn grade_details(&self, grade: &Grade) -> Result<GradeDetails> {
        Ok(self.grades.details(&grade).await?)
    }

    pub async fn messages_recieved(
        &self,
        page: Page,
        limit: Limit,
    ) -> Result<RecievedMessagePreviews> {
        Ok(self
            .synergia_api
            .messages()
            .fetch_recieved(page, limit)
            .await?)
    }
}
