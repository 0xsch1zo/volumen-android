use std::sync::Arc;

use crate::{
    net::{
        synergia_api::{AuthenticatedState, Message},
        SynergiaApi,
    },
    repositories::{
        grades::{Grade, GradeDetails, GradesRepository},
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

    pub async fn messages_recieved(&self) -> Result<Vec<Message>> {
        Ok(self.synergia_api.messages().fetch_recieved().await?)
    }
}
