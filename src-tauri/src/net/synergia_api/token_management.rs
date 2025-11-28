use reqwest::{multipart, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;
use tokio::sync::{RwLock, RwLockReadGuard};
use url::Url;

use crate::{
    net::{
        synergia_api::{
            internal_types::{
                PortalAccessToken, PortalRefreshToken, PortalTokenPair, SynergiaTokens,
            },
            LIBRUS_API_URL, PORTAL_URL, SYNERGIA_URL,
        },
        ErrorStatusMiddleware, IsSameBaseExt,
    },
    repositories::entities::SynergiaUserId,
};

#[derive(Error, Debug)]
pub enum TokenManagerError {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest middleware error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}

#[repr(transparent)]
pub struct AuthCode<'a>(&'a str);

impl<'a> AuthCode<'a> {
    pub fn new(code: &'a str) -> Self {
        Self(code)
    }
}

enum PortalGrant<'a> {
    RefreshToken(&'a PortalRefreshToken),
    AuthCode(&'a str),
}

impl<'a> From<AuthCode<'a>> for PortalGrant<'a> {
    fn from(value: AuthCode<'a>) -> Self {
        PortalGrant::AuthCode(value.0)
    }
}

#[derive(Debug, Clone)]
pub struct Tokens {
    pub portal_token_pair: PortalTokenPair,
    pub synergia_tokens: SynergiaTokens,
}

#[derive(Debug)]
pub struct TokenManager {
    tokens: RwLock<Tokens>,
    client: ClientWithMiddleware,
}

impl TokenManager {
    pub async fn with_authorized(code: AuthCode<'_>) -> Result<Self, TokenManagerError> {
        let client = ClientBuilder::new(Client::new())
            .with(ErrorStatusMiddleware)
            .build();
        let portal_token_pair = Self::fetch_portal_token(&client, code.into()).await?;
        let synergia_tokens =
            Self::fetch_synergia_tokens(&client, &portal_token_pair.access_token).await?;

        Ok(Self {
            tokens: RwLock::new(Tokens {
                synergia_tokens,
                portal_token_pair,
            }),
            client,
        })
    }

    pub async fn get(&self) -> RwLockReadGuard<'_, Tokens> {
        self.tokens.read().await
    }

    // this function ensures that token refreshes happen atomically
    // TODO: fix this garbage, the lock is held during a network request for no reason
    pub async fn refresh(&self) -> Result<(), TokenManagerError> {
        let mut tokens = self.tokens.write().await;

        tokens.portal_token_pair = Self::fetch_portal_token(
            &self.client,
            PortalGrant::RefreshToken(&tokens.portal_token_pair.refresh_token),
        )
        .await?;

        tokens.synergia_tokens =
            Self::fetch_synergia_tokens(&self.client, &tokens.portal_token_pair.access_token)
                .await?;

        drop(tokens);
        Ok(())
    }

    async fn fetch_portal_token(
        client: &ClientWithMiddleware,
        grant: PortalGrant<'_>,
    ) -> Result<PortalTokenPair, TokenManagerError> {
        const ACCESS_TOKEN_ENDPOINT: &str = "/oauth2/access_token";
        const CLIENT_ID: &str = "VaItV6oRutdo8fnjJwysnTjVlvaswf52ZqmXsJGP";

        let grant_type = match &grant {
            PortalGrant::RefreshToken(_) => "refresh_token",
            PortalGrant::AuthCode(_) => "authorization_code",
        };

        let grant_column_name = match &grant {
            PortalGrant::RefreshToken(_) => "refresh_token",
            PortalGrant::AuthCode(_) => "code",
        };

        let grant = match grant {
            PortalGrant::RefreshToken(token) => token.as_inner(),
            PortalGrant::AuthCode(code) => code,
        }
        .to_owned();

        let form = multipart::Form::new()
            .text("grant_type", grant_type)
            .text("client_id", CLIENT_ID)
            .text("redirect_uri", "app://librus")
            .text(grant_column_name, grant);

        let tokens = client
            .post(PORTAL_URL.join(ACCESS_TOKEN_ENDPOINT).unwrap())
            .multipart(form)
            .send()
            .await?
            .json::<PortalTokenPair>()
            .await?;
        Ok(tokens)
    }

    async fn fetch_synergia_tokens(
        client: &ClientWithMiddleware,
        portal_access_token: &PortalAccessToken,
    ) -> Result<SynergiaTokens, TokenManagerError> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";

        let synergia_tokens = client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .await?
            .json::<SynergiaTokens>()
            .await?;
        Ok(synergia_tokens)
    }
}

#[derive(Error, Debug)]
pub enum TokenPickerError {
    #[error("synergia access token not found")]
    SynergiaAccessTokenNotFound,
}

#[derive(Debug)]
pub struct TokenPicker {
    synergia_id: SynergiaUserId,
}

impl TokenPicker {
    pub fn new(synergia_id: SynergiaUserId) -> Self {
        Self { synergia_id }
    }

    pub fn pick(&self, url: &Url, tokens: &Tokens) -> Result<Option<String>, TokenPickerError> {
        let synergia_token = tokens
            .synergia_tokens
            .inner()
            .get(&self.synergia_id)
            .ok_or(TokenPickerError::SynergiaAccessTokenNotFound)?
            .as_inner();

        let managed_hosts = [
            (PORTAL_URL, tokens.portal_token_pair.access_token.as_inner()),
            (SYNERGIA_URL, synergia_token),
            (LIBRUS_API_URL, synergia_token),
        ];

        let Some(token) = managed_hosts
            .into_iter()
            .find(|other| url.is_same_base(&other.0))
        else {
            return Ok(None);
        };
        Ok(Some(token.1.to_owned()))
    }
}
