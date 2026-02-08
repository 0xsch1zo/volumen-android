use thiserror::Error;

use crate::{
    error::{StatefulError, StatefulResultExt},
    net::{
        synergia_api::{UnauthenticatedState, UnauthenticatedSynergiaApiError},
        SynergiaApi,
    },
    repositories::account_selection::AccountSelectionRepository,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to construct unauthenticated synergia api")]
    UnauthedSynergiaConstructionError(#[source] UnauthenticatedSynergiaApiError),
    #[error("login error")]
    LoginError(#[source] UnauthenticatedSynergiaApiError),
}

#[derive(Debug)]
pub struct LoginRepository {
    synergia_api: SynergiaApi<UnauthenticatedState>,
}

impl LoginRepository {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            synergia_api: SynergiaApi::try_new()
                .map_err(Error::UnauthedSynergiaConstructionError)?,
        })
    }

    pub async fn login(
        self,
        email: String,
        password: String,
    ) -> Result<AccountSelectionRepository, StatefulError<Self, Error>> {
        self.synergia_api
            .login(email, password)
            .await
            .map(AccountSelectionRepository::new)
            .map_err_state(|s| LoginRepository { synergia_api: s })
            .map_stateful_err(Error::LoginError)
            .map_stateful_err(Into::into)
    }
}
