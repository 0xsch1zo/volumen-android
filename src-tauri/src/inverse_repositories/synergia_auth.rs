use std::{collections::HashMap, future::Future, sync::Arc};

use serde::Serialize;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    net::synergia_api::token_management::{TokensApi, TokensApiError},
    repositories::account_selection::SynergiaUserId,
    sync::SingleParallelFlight,
};

#[derive(Error, Clone, Debug)]
pub enum Error {
    #[error("synergia token not found for id: {0:?}")]
    SynergiaTokenNotFound(SynergiaUserId),
    #[error("token fetch error")]
    TokenFetchError(#[source] TokensApiError),
    #[error("token refresh error")]
    TokenRefreshError(#[source] TokensApiError),
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PortalAccessToken(String);

impl PortalAccessToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PortalRefreshToken(String);

impl PortalRefreshToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PortalTokenPair {
    pub access_token: PortalAccessToken,
    pub refresh_token: PortalRefreshToken,
}

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct SynergiaToken(String);

impl SynergiaToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

type SynergiaTokens = HashMap<SynergiaUserId, SynergiaToken>;

#[derive(Debug)]
pub struct Tokens {
    pub portal_token_pair: PortalTokenPair,
    pub synergia_tokens: SynergiaTokens,
}

pub struct SynergiaAuthInvRepository {
    synergia_user_id: SynergiaUserId,
    tokens: RwLock<Option<Tokens>>,
    tokens_api: TokensApi,
    refresh_worker: SingleParallelFlight<Result<(), Error>>,
}

impl SynergiaAuthInvRepository {
    //pub(super) async fn try_from_authcode() -> Self {}

    pub async fn authenciation(&self) -> Result<SynergiaToken, Error> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .as_ref()
            .expect("tokens should be none only on refresh")
            .synergia_tokens
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
}
