use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use thiserror::Error;

use crate::{
    error::StatefulResultExt,
    net::{
        synergia_api::{
            auth_manager::AuthorizationManager,
            token_management::{TokenManager, TokenManagerError, TokenPicker, TokenPickerError},
            AuthenticatedState, StatefulError, PORTAL_URL,
        },
        ErrorStatusMiddleware, SynergiaApi,
    },
    repositories::{SynergiaAccounts, SynergiaUserId},
};

#[derive(Error, Debug)]
pub enum AccountSelectorError {
    #[error("token picker error")]
    PickerError(#[from] TokenPickerError),
    #[error("token manager error")]
    TokenManagerError(#[from] TokenManagerError),
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}

type Result<T, E = AccountSelectorError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct AccountSelector {
    token_manager: TokenManager,
}

impl AccountSelector {
    pub fn new(token_manager: TokenManager) -> Self {
        Self { token_manager }
    }

    pub fn select(
        self,
        id: SynergiaUserId,
    ) -> Result<SynergiaApi<AuthenticatedState>, StatefulError<Self>> {
        let auth_manager = AuthorizationManager::new(self.token_manager, TokenPicker::new(id));
        SynergiaApi::<AuthenticatedState>::try_from_auth_manager(auth_manager)
            .map_state(|s| Self::new(s.into()))
    }

    pub async fn accounts(&self) -> Result<SynergiaAccounts> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";
        let portal_access_token = &self
            .token_manager
            .get()
            .await
            .portal_token_pair
            .access_token;

        let client = Client::builder().connection_verbose(true).build()?;
        let client = ClientBuilder::new(client)
            .with(ErrorStatusMiddleware)
            .build();

        let accounts = client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .await?
            .json::<SynergiaAccounts>()
            .await?;
        Ok(accounts)
    }
}
