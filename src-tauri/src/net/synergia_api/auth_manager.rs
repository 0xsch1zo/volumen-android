use std::sync::Arc;

use log::debug;
use reqwest::{
    header::{self, ToStrError},
    redirect::Policy,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tauri::http::HeaderValue;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    net::{
        self,
        synergia_api::{
            api::auth::{PowerCookie, SynergiaToken, SynergiaUserId, Tokens},
            token_management::{TokensApi, TokensApiError},
            AuthenticatedSynergiaEndpoints, MessagesEndpoints,
        },
        ResponseCookieExt, ResponseCookieExtError,
    },
    sync::SingleParallelFlight,
};

#[derive(Error, Clone, Debug)]
pub enum ManagerError {
    #[error("synergia token not found for id: {0:?}")]
    SynergiaTokenNotFound(SynergiaUserId),
    #[error("token refresh error")]
    TokenRefreshError(#[source] TokensApiError),
}

#[derive(Debug)]
pub struct AuthorizationManager {
    synergia_user_id: SynergiaUserId,
    tokens: RwLock<Option<Tokens>>,
    tokens_api: TokensApi,
    refresh_worker: SingleParallelFlight<Result<(), ManagerError>>,
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

    pub async fn authenciation(&self) -> Result<SynergiaToken, ManagerError> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .as_ref()
            .expect("tokens should be none only on refresh")
            .synergia_tokens
            .inner()
            .get(&self.synergia_user_id)
            .ok_or(ManagerError::SynergiaTokenNotFound(self.synergia_user_id))?
            .to_owned())
    }

    pub async fn refresh(&self) -> Result<(), ManagerError> {
        self.refresh_worker
            .work(async || {
                let mut tokens_guard = self.tokens.write().await;
                let tokens = tokens_guard.take().unwrap();
                *tokens_guard = Some(
                    self.tokens_api
                        .refresh(tokens)
                        .await
                        .map_err(ManagerError::TokenRefreshError)?,
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

#[derive(Error, Debug)]
pub enum MessagesManagerError {
    #[error("failed to initialize network client of messages auth manager")]
    ClientInitFailed(#[source] reqwest::Error),
    #[error("failed to get authentication for power cookie fetch")]
    PowerCookieRequestAuthentictationFailure(#[source] ManagerError),
    #[error("failed to get power cookies")]
    PowerCookieAcquisitionFailure(#[source] reqwest_middleware::Error),
    #[error("power cookie not found in response headers of /Me")]
    PowerCookieNotFound,
    #[error("failed to convert cookie header to &str, while acquiring the power cookie")]
    CookieHeaderToStrError(#[source] ToStrError),
    #[error("cookie parsing error, while acquiring power cookie")]
    CookieParseError(#[source] cookie::ParseError),
    #[error("failed to parse power cookie from raw cookie")]
    PowerCookieParseError(#[source] cookie_store::CookieError),
    #[error("cookie extraction error")]
    CookieExtractError(#[source] ResponseCookieExtError),
}

#[derive(Debug, Clone, Copy)]
enum MessagesModuleAuthorizationState {
    Authorized,
    Unauthorized,
}

#[derive(Debug)]
pub enum MessagesModuleAuthorizationResponse {
    AlreadyAuthorized,
    Authorization(PowerCookie),
}

#[derive(Debug)]
pub struct MessagesAuthManager {
    auth_manager: Arc<AuthorizationManager>,
    authorization_state: RwLock<MessagesModuleAuthorizationState>,
    client: ClientWithMiddleware,
}

impl MessagesAuthManager {
    pub fn try_new(auth_manager: Arc<AuthorizationManager>) -> Result<Self, MessagesManagerError> {
        let client = ClientBuilder::new(
            net::default_client_options()
                .connection_verbose(true)
                .redirect(Self::redirect_policy())
                .build()
                .map_err(MessagesManagerError::ClientInitFailed)?,
        )
        .build();
        let authorization_state = RwLock::new(MessagesModuleAuthorizationState::Unauthorized);
        Ok(Self {
            authorization_state,
            auth_manager,
            client,
        })
    }

    pub async fn request_authoriztaion(
        &self,
        power_cookie: Option<PowerCookie>,
    ) -> Result<MessagesModuleAuthorizationResponse, MessagesManagerError> {
        let state = *self.authorization_state.read().await;
        match state {
            MessagesModuleAuthorizationState::Authorized => {
                Ok(MessagesModuleAuthorizationResponse::AlreadyAuthorized)
            }
            MessagesModuleAuthorizationState::Unauthorized => {
                let power_cookie = MessagesModuleAuthorizationResponse::Authorization(
                    self.authorize(power_cookie).await?,
                );
                *self.authorization_state.write().await =
                    MessagesModuleAuthorizationState::Authorized;
                Ok(power_cookie)
            }
        }
    }

    fn redirect_policy() -> Policy {
        Policy::custom(|attempt| {
            debug!("following redirect: {}", attempt.url());
            attempt.follow()
        })
    }

    // the goddamn cookie api is complete and utter dogshit
    async fn authorize(
        &self,
        power_cookie: Option<PowerCookie>,
    ) -> Result<PowerCookie, MessagesManagerError> {
        let power_cookie = match power_cookie {
            Some(c) => c,
            None => self.fetch_power_cookie().await?,
        };

        let authentication = self
            .auth_manager
            .authenciation()
            .await
            .map(SynergiaToken::into_cookie_string)
            .map_err(MessagesManagerError::PowerCookieRequestAuthentictationFailure)?;

        self.client
            .get(AuthenticatedSynergiaEndpoints::Messages(MessagesEndpoints::Authorization).url())
            .header(
                header::COOKIE,
                &format!("{authentication}; {}", power_cookie.to_cookie_string()),
            )
            .send()
            .await
            .map_err(MessagesManagerError::PowerCookieAcquisitionFailure)?;

        Ok(power_cookie)
    }

    async fn fetch_power_cookie(&self) -> Result<PowerCookie, MessagesManagerError> {
        let authentication = self
            .auth_manager
            .authenciation()
            .await
            .map(SynergiaToken::into_cookie_string)
            .map_err(MessagesManagerError::PowerCookieRequestAuthentictationFailure)?;

        self.client
            .get(AuthenticatedSynergiaEndpoints::Me.url())
            .header(header::COOKIE, &authentication)
            .send()
            .await
            .map_err(MessagesManagerError::PowerCookieAcquisitionFailure)?
            .extract_cookie(PowerCookie::NAME)
            .map_err(MessagesManagerError::CookieExtractError)?
            .ok_or(MessagesManagerError::PowerCookieNotFound)
            .map(cookie::Cookie::into_owned)
            .map(PowerCookie::new)
    }
}
