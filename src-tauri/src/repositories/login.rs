use crate::{
    error::StatefulResultExt,
    net::{synergia_api::UnauthenticatedState, SynergiaApi},
    repositories::{account_selection::AccountSelectionRepository, Result, StatefulError},
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
        email: &str,
        password: &str,
    ) -> Result<AccountSelectionRepository, StatefulError<Self>> {
        self.synergia_api
            .login(email, password)
            .await
            .map(AccountSelectionRepository::new)
            .map_err_state(|s| LoginRepository { synergia_api: s })
            .map_stateful_err(Into::into)
    }
}
