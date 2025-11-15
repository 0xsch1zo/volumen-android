use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use thiserror::Error;
use url::Url;

use crate::net::{
    synergia_api::{
        private_types::{SynergiaAccounts, SynergiaUserId},
        token_management::{TokenManager, TokenManagerError, TokenPicker, TokenPickerError},
        PORTAL_URL,
    },
    ErrorStatusMiddleware,
};

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error("token picker error")]
    PickerError(#[from] TokenPickerError),
    #[error("token manager error")]
    TokenManagerError(#[from] TokenManagerError),
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}

type Result<T, E = AccountManagerError> = std::result::Result<T, E>;

pub trait AccountManagerState {}

#[derive(Debug)]
pub struct UnselectedAccountState {
    token_manager: TokenManager,
}

#[derive(Debug)]
pub struct SelectedAccountState {
    token_manager: TokenManager,
    token_picker: TokenPicker,
}

impl AccountManagerState for UnselectedAccountState {}
impl AccountManagerState for SelectedAccountState {}

#[derive(Debug)]
pub struct AccountManager<S: AccountManagerState = UnselectedAccountState> {
    state: S,
}

impl AccountManager<UnselectedAccountState> {
    pub fn new(token_manager: TokenManager) -> Self {
        Self {
            state: UnselectedAccountState { token_manager },
        }
    }

    pub fn select(self, id: SynergiaUserId) -> AccountManager<SelectedAccountState> {
        AccountManager::<SelectedAccountState> {
            state: SelectedAccountState {
                token_manager: self.state.token_manager,
                token_picker: TokenPicker::new(id),
            },
        }
    }

    pub async fn accounts(&self) -> Result<SynergiaAccounts> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";
        let portal_access_token = &self
            .state
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

impl AccountManager<SelectedAccountState> {
    pub async fn managed_token(&self, url: &Url) -> Result<Option<String>> {
        let tokens = self.state.token_manager.get().await;
        Ok(self.state.token_picker.pick(url, &tokens)?)
    }

    pub async fn is_managed(&self, url: &Url) -> Result<bool> {
        let tokens = self.state.token_manager.get().await;
        Ok(self.state.token_picker.pick(url, &tokens)?.is_some())
    }

    pub async fn refresh(&self) -> Result<()> {
        Ok(self.state.token_manager.refresh().await?)
    }
}
