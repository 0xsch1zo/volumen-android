use std::sync::Arc;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        grades::{Grade, GradeDetails, GradesRepository},
        Result,
    },
};

#[derive(Debug)]
pub struct MainRepository {
    grades: GradesRepository,
}

impl MainRepository {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        let synergia_api = Arc::new(synergia_api);

        let grades = GradesRepository::new(Arc::clone(&synergia_api));

        Self { grades }
    }

    /*pub async fn me(&self) -> Result<String> {
        Ok(self.synergia_api.me().await?)
    }*/

    pub async fn grades(&self) -> Result<Vec<Grade>> {
        Ok(self.grades.grades().await?)
    }

    pub async fn grade_details(&self, grade: &Grade) -> Result<GradeDetails> {
        Ok(self.grades.details(&grade).await?)
    }
}
