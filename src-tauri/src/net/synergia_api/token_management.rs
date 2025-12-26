use std::sync::Arc;

use futures::TryFutureExt;
use reqwest::multipart;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;

use crate::net::{
    self,
    synergia_api::{
        api::auth::{
            AuthCode, PortalAccessToken, PortalRefreshToken, PortalTokenPair, SynergiaTokens,
            Tokens,
        },
        PORTAL_URL,
    },
    ErrorStatusMiddleware,
};

#[derive(Error, Debug, Clone)]
pub enum TokenFetchError {
    #[error("reqwest error")]
    ReqwestError(#[from] Arc<reqwest::Error>),
    #[error("reqwest middleware error")]
    ReqwestMiddlewareError(#[from] Arc<reqwest_middleware::Error>),
}

// experimental error handling solution
#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct PortalTokenFetchError(#[from] TokenFetchError);

#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct SynergiaTokenFetchError(#[from] TokenFetchError);

#[derive(Error, Debug, Clone)]
pub enum TokensApiError {
    #[error("falied to fetch portal tokesn")]
    PortalTokenError(#[from] PortalTokenFetchError),
    #[error("falied to fetch synergia tokens")]
    SynergiaTokenError(#[from] SynergiaTokenFetchError),
}

enum PortalGrant {
    RefreshToken(PortalRefreshToken),
    AuthCode(AuthCode),
}

#[derive(Debug)]
pub struct TokensApi {
    client: ClientWithMiddleware,
}

impl TokensApi {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new(net::default_client_options().build().expect("temp")) // FIXME
                .with(ErrorStatusMiddleware)
                .build(),
        }
    }

    async fn fetch_portal_token(
        &self,
        grant: &PortalGrant,
    ) -> Result<PortalTokenPair, PortalTokenFetchError> {
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
            PortalGrant::AuthCode(code) => code.as_inner(),
        }
        .to_owned();

        let form = multipart::Form::new()
            .text("grant_type", grant_type)
            .text("client_id", CLIENT_ID)
            .text("redirect_uri", "app://librus")
            .text(grant_column_name, grant);

        let tokens = self
            .client
            .post(PORTAL_URL.join(ACCESS_TOKEN_ENDPOINT).unwrap())
            .multipart(form)
            .send()
            .map_err(|e| TokenFetchError::from(Arc::new(e)))
            .await?
            .json::<PortalTokenPair>()
            .map_err(|e| TokenFetchError::from(Arc::new(e)))
            .await?;
        Ok(tokens)
    }

    async fn fetch_synergia_tokens(
        &self,
        portal_access_token: &PortalAccessToken,
    ) -> Result<SynergiaTokens, PortalTokenFetchError> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";

        let synergia_tokens = self
            .client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .map_err(|e| TokenFetchError::from(Arc::new(e)))
            .await?
            .json::<SynergiaTokens>()
            .map_err(|e| TokenFetchError::from(Arc::new(e)))
            .await?;
        Ok(synergia_tokens)
    }

    pub(super) async fn fetch_tokens(&self, code: AuthCode) -> Result<Tokens, TokensApiError> {
        let portal_token_pair = self
            .fetch_portal_token(&PortalGrant::AuthCode(code))
            .await?;
        let synergia_tokens = self
            .fetch_synergia_tokens(&portal_token_pair.access_token)
            .await?;

        Ok(Tokens {
            synergia_tokens,
            portal_token_pair,
        })
    }

    pub async fn refresh(&self, tokens: Tokens) -> Result<Tokens, TokensApiError> {
        let portal_token_pair = self
            .fetch_portal_token(&PortalGrant::RefreshToken(
                tokens.portal_token_pair.refresh_token,
            ))
            .await?;

        let synergia_tokens = self
            .fetch_synergia_tokens(&portal_token_pair.access_token)
            .await?;

        Ok(Tokens {
            synergia_tokens,
            portal_token_pair,
        })
    }
}
