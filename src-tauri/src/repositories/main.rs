use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::Result,
};

#[derive(Debug)]
pub struct MainRepo {
    synergia_api: SynergiaApi<AuthenticatedState>,
}

impl MainRepo {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    pub async fn me(&self) -> Result<String> {
        Ok(self.synergia_api.me().await?)
    }
}
