use thiserror::Error;
use url::Url;

use crate::net::synergia_api::{
    private_types::SynergiaUserId,
    token_management::{TokenManager, TokenManagerError, TokenPicker, TokenPickerError},
};

#[derive(Error, Debug)]
pub enum AccountManagementError {
    #[error("token picker error")]
    PickerError(#[from] TokenPickerError),
    #[error("token manager error")]
    TokenManagerError(#[from] TokenManagerError),
}

trait AccountManagerState {}

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
}

impl AccountManager<SelectedAccountState> {
    pub async fn managed_token(&self, url: &Url) -> Result<Option<String>, AccountManagementError> {
        let tokens = self.state.token_manager.get().await;
        Ok(self.state.token_picker.pick(url, &tokens)?)
    }

    pub async fn is_managed(&self, url: &Url) -> Result<bool, AccountManagementError> {
        let tokens = self.state.token_manager.get().await;
        Ok(self.state.token_picker.pick(url, &tokens)?.is_some())
    }

    pub async fn refresh(&self) -> Result<(), AccountManagementError> {
        Ok(self.state.token_manager.refresh().await?)
    }
}
