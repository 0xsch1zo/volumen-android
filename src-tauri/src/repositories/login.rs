use crate::{
    error::StatefulResultExt,
    net::{
        synergia_api::{AuthenticatedState, UnauthenticatedState},
        SynergiaApi,
    },
    repositories::{
        account_selection::AccountSelectionRepository, MainRepository, Result, StatefulError,
    },
};

#[derive(Debug)]
pub struct LoginRepository {
    synergia_api: SynergiaApi<UnauthenticatedState>,
}

impl LoginRepository {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            synergia_api: SynergiaApi::try_new()?,
        })
    }

    pub async fn login(
        self,
        email: String,
        password: String,
    ) -> Result<MainRepository, StatefulError<Self>> {
        let synergia_api = self
            .synergia_api
            .new_login(email, password)
            .await
            .expect("fix later"); // yeah this sounds very safe

        Ok(MainRepository::new(synergia_api))
    }
}
