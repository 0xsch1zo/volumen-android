use thiserror::Error;
use url::Url;

use crate::net::synergia_api::token_management::{
    PickedToken, TokenManager, TokenManagerError, TokenPicker, TokenPickerError,
};

#[derive(Error, Debug)]
pub enum AuthorizationManagerError {
    #[error("token picker error")]
    PickerError(#[from] TokenPickerError),
    #[error("token manager error")]
    TokenManagerError(#[from] TokenManagerError),
}

type Result<T, E = AuthorizationManagerError> = std::result::Result<T, E>;

pub struct AuthorizationManager {
    token_manager: TokenManager,
    token_picker: TokenPicker,
}

impl AuthorizationManager {
    pub fn new(token_manager: TokenManager, token_picker: TokenPicker) -> Self {
        Self {
            token_manager,
            token_picker,
        }
    }

    pub async fn managed_token(&self, url: &Url) -> Result<Option<PickedToken>> {
        let tokens = self.token_manager.get().await;
        Ok(self.token_picker.pick(url, &tokens)?)
    }

    pub async fn is_managed(&self, url: &Url) -> Result<bool> {
        let tokens = self.token_manager.get().await;
        Ok(self.token_picker.pick(url, &tokens)?.is_some())
    }

    pub async fn refresh(&self) -> Result<()> {
        Ok(self.token_manager.refresh().await?)
    }
}

impl From<AuthorizationManager> for TokenManager {
    fn from(value: AuthorizationManager) -> Self {
        value.token_manager
    }
}
