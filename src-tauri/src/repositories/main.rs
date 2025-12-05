use std::sync::Arc;

use crate::{
    cache::Cache,
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        grades::{Grade, GradesRepository},
        subjects::SubjectsRepository,
        users::UsersRepository,
        Result,
    },
};

#[derive(Debug)]
pub struct MainRepository {
    grades: GradesRepository,
    subjects: SubjectsRepository,
    users: UsersRepository,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: Arc<Cache>,
}

impl MainRepository {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        let synergia_api = Arc::new(synergia_api);
        let cache = Arc::new(Cache::new());

        let grades = GradesRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));
        let subjects = SubjectsRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));
        let users = UsersRepository::new(Arc::clone(&synergia_api), Arc::clone(&cache));

        Self {
            grades,
            subjects,
            users,
            synergia_api,
            cache,
        }
    }

    pub async fn me(&self) -> Result<String> {
        Ok(self.synergia_api.me().await?)
    }

    pub async fn grades(&self) -> Result<Vec<Grade>> {
        Ok(self.grades.grades().await?)
    }
}
