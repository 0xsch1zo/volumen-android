use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    net::synergia_api::{
        api::auth::{SynergiaToken, SynergiaUserId, Tokens},
        token_management::{TokensApi, TokensApiError},
    },
    sync::SingleParallelFlight,
};

#[derive(Error, Clone, Debug)]
pub enum Error {
    #[error("synergia token not found for id: {0:?}")]
    SynergiaTokenNotFound(SynergiaUserId),
    #[error("token refresh error")]
    TokenRefreshError(#[source] TokensApiError),
}

pub struct AuthorizationManager {
    synergia_user_id: SynergiaUserId,
    tokens: RwLock<Option<Tokens>>,
    tokens_api: TokensApi,
    refresh_worker: SingleParallelFlight<Result<(), Error>>,
}

impl AuthorizationManager {
    pub fn new(tokens: Tokens, tokens_api: TokensApi, synergia_user_id: SynergiaUserId) -> Self {
        Self {
            synergia_user_id,
            tokens: RwLock::new(Some(tokens)),
            tokens_api,
            refresh_worker: SingleParallelFlight::new(),
        }
    }

    pub async fn authenciation(&self) -> Result<SynergiaToken, Error> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .as_ref()
            .expect("tokens should be none only on refresh")
            .synergia_tokens
            .inner()
            .get(&self.synergia_user_id)
            .ok_or(Error::SynergiaTokenNotFound(self.synergia_user_id))?
            .to_owned())
    }

    pub async fn refresh(&self) -> Result<(), Error> {
        self.refresh_worker
            .work(async || {
                let mut tokens_guard = self.tokens.write().await;
                let tokens = tokens_guard.take().unwrap();
                *tokens_guard = Some(
                    self.tokens_api
                        .refresh(tokens)
                        .await
                        .map_err(Error::TokenRefreshError)?,
                );
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub fn decay(self) -> (Tokens, TokensApi) {
        (self.tokens.into_inner().unwrap(), self.tokens_api)
    }
}
