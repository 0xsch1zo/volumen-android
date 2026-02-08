use std::sync::Arc;

use futures::TryFutureExt;
use reqwest::{multipart, redirect::Policy, Request, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use tauri::http::Extensions;
use thiserror::Error;

use crate::net::{
    self,
    synergia_api::{
        api::auth::{
            AuthCode, AutoLoginResponse, LibrusApiToken, LibrusApiTokens, PortalAccessToken,
            PortalRefreshToken, PortalTokenPair, PowerCookie, SynergiaToken, SynergiaUserId,
        },
        LIBRUS_API_URL, PORTAL_URL, SYNERGIA_URL,
    },
    ErrorStatusMiddleware, RequestCookieExt, RequestCookieExtError, ResponseCookieExt,
    ResponseCookieExtError,
};

#[derive(Error, Debug, Clone)]
pub enum CredentialFetchError {
    #[error("json deserialization error")]
    JsonDeserializationError(#[source] Arc<reqwest::Error>),
    #[error("failed to send request")]
    RequestSendError(#[source] Arc<reqwest_middleware::Error>),
}

// experimental error handling solution
#[derive(Error, Debug, Clone)]
#[error(transparent)]
pub struct PortalCredentialFetchError(#[from] CredentialFetchError);

#[derive(Error, Debug, Clone)]
pub enum LibrusApiCredentialFetchError {
    #[error("synergia account not found")]
    SynergiaAcocuntNotFound,
    #[error("general credential fetch error")]
    GeneralCredentialFetchError(#[from] CredentialFetchError),
}

#[derive(Error, Debug, Clone)]
pub enum SynergiaCredentialExtractionError {
    #[error("synergia token not found in login response")]
    SynergiaTokenNotFound,
    #[error("cookie extraction error")]
    CookieExtractionError(#[source] ResponseCookieExtError),
    #[error("power cookie not found in login response")]
    PowerCookieNotFound,
}

#[derive(Error, Debug, Clone)]
pub enum SynergiaCredentialFetchError {
    #[error("general credential fetch error")]
    GeneralCredentialFetchError(#[from] CredentialFetchError),
    #[error("credentail extraction error")]
    CredentialExtractionError(#[source] SynergiaCredentialExtractionError),
}

#[derive(Error, Debug, Clone)]
pub enum SynergiaCredentialRefreshError {
    #[error("credentail extraction error")]
    CredentialExtractionError(#[source] SynergiaCredentialExtractionError),
    #[error("refresh request construction error")]
    RefreshRequestConstructionError(#[source] Arc<reqwest::Error>),
    #[error("cookie insert error")]
    CookieInsertError(#[source] RequestCookieExtError),
    #[error("failed to send refresh request")]
    RefreshRequestSendError(#[source] Arc<reqwest_middleware::Error>),
}

#[derive(Error, Debug, Clone)]
pub enum CredentialManagerError {
    #[error("falied to fetch portal tokens")]
    PortalTokenError(#[from] PortalCredentialFetchError),
    #[error("falied to fetch librus api token")]
    LibrusApiTokenError(#[from] LibrusApiCredentialFetchError),
    #[error("falied to fetch synergia tokens")]
    SynergiaTokenError(#[from] SynergiaCredentialFetchError),
    #[error("client construction error")]
    ClientConstructionError(#[source] Arc<reqwest::Error>),
}

#[derive(Error, Debug)]
#[error("unauthenticated to fetch credentials")]
pub struct UnauthenticatedError;

#[derive(Error, Debug)]
#[error("failed to construt client of PortalCredentialManager")]
pub struct PortalClientConstructionError(#[source] reqwest::Error);

struct UnauthenticatedErrorMiddleware;

#[async_trait::async_trait]
impl Middleware for UnauthenticatedErrorMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let res = next.run(req, extensions).await?;

        match res.status() {
            StatusCode::UNAUTHORIZED => {
                Err(reqwest_middleware::Error::middleware(UnauthenticatedError))
            }
            _ => Ok(res),
        }
    }
}

enum PortalGrant<'a> {
    RefreshToken(PortalRefreshToken),
    AuthCode(&'a AuthCode),
}

#[derive(Debug, Clone)]
pub struct PortalCredentialManager {
    client: Arc<ClientWithMiddleware>,
}

impl PortalCredentialManager {
    pub fn try_new() -> Result<Self, PortalClientConstructionError> {
        Ok(Self {
            client: Arc::new(
                ClientBuilder::new(
                    net::default_client_options()
                        .build()
                        .map_err(PortalClientConstructionError)?,
                )
                .build(),
            ),
        })
    }

    // for internal use wtih main CredentialManagerk
    fn new(client: Arc<ClientWithMiddleware>) -> Self {
        Self { client }
    }

    pub async fn fetch_from_authcode(
        &self,
        code: &AuthCode,
    ) -> Result<PortalTokenPair, PortalCredentialFetchError> {
        self.fetch(&PortalGrant::AuthCode(code)).await
    }

    async fn fetch(
        &self,
        grant: &PortalGrant<'_>,
    ) -> Result<PortalTokenPair, PortalCredentialFetchError> {
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
            .map_err(|e| CredentialFetchError::RequestSendError(Arc::new(e)))
            .await?
            .json::<PortalTokenPair>()
            .map_err(|e| CredentialFetchError::JsonDeserializationError(Arc::new(e)))
            .await?;
        Ok(tokens)
    }
}

#[derive(Clone, Debug)]
struct LibrusApiCredentialManager {
    client: Arc<ClientWithMiddleware>,
    user_id: SynergiaUserId,
}

impl LibrusApiCredentialManager {
    fn new(client: Arc<ClientWithMiddleware>, user_id: SynergiaUserId) -> Self {
        Self { client, user_id }
    }

    async fn fetch(
        &self,
        portal_access_token: &PortalAccessToken,
    ) -> Result<LibrusApiToken, LibrusApiCredentialFetchError> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";

        let librus_api_tokens = self
            .client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .map_err(|e| CredentialFetchError::RequestSendError(Arc::new(e)))
            .await?
            .json::<LibrusApiTokens>()
            .map_err(|e| CredentialFetchError::JsonDeserializationError(Arc::new(e)))
            .await?;
        Ok(librus_api_tokens
            .into_inner()
            .remove(&self.user_id)
            .ok_or(LibrusApiCredentialFetchError::SynergiaAcocuntNotFound)?)
    }
}

// sits here because it's not a direct part of the api itself
#[derive(Clone, Debug)]
pub struct SynergiaCredentials {
    pub token: SynergiaToken,
    pub power_cookie: PowerCookie,
}

impl SynergiaCredentials {
    fn extract_from_response(resp: &Response) -> Result<Self, SynergiaCredentialExtractionError> {
        let power_cookie = resp
            .extract_cookie(PowerCookie::NAME)
            .map_err(SynergiaCredentialExtractionError::CookieExtractionError)?
            .map(cookie::Cookie::into_owned)
            .map(PowerCookie::new)
            .ok_or(SynergiaCredentialExtractionError::PowerCookieNotFound)?;

        let token = resp
            .extract_cookie(SynergiaToken::NAME)
            .map_err(SynergiaCredentialExtractionError::CookieExtractionError)?
            .map(cookie::Cookie::into_owned)
            .map(SynergiaToken::new)
            .ok_or(SynergiaCredentialExtractionError::SynergiaTokenNotFound)?;

        Ok(Self {
            token,
            power_cookie,
        })
    }

    fn extract_from_response_with_powercookie(
        resp: &Response,
        power_cookie: PowerCookie,
    ) -> Result<Self, SynergiaCredentialExtractionError> {
        let token = resp
            .extract_cookie(SynergiaToken::NAME)
            .map_err(SynergiaCredentialExtractionError::CookieExtractionError)?
            .map(cookie::Cookie::into_owned)
            .map(SynergiaToken::new)
            .ok_or(SynergiaCredentialExtractionError::SynergiaTokenNotFound)?;

        Ok(Self {
            token,
            power_cookie,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SynergiaCredentialManager {
    client: Arc<ClientWithMiddleware>,
}

impl SynergiaCredentialManager {
    fn new(client: Arc<ClientWithMiddleware>) -> Self {
        Self { client }
    }

    async fn fetch(
        &self,
        librus_api_token: &LibrusApiToken,
    ) -> Result<SynergiaCredentials, SynergiaCredentialFetchError> {
        const AUTO_LOGIN_TOKEN_ENDPOINT: &str = "/2.0/AutoLoginToken";
        let auto_login_token = self
            .client
            .post(LIBRUS_API_URL.join(AUTO_LOGIN_TOKEN_ENDPOINT).unwrap())
            .bearer_auth(librus_api_token.as_inner())
            .send()
            .await
            .map_err(|e| CredentialFetchError::RequestSendError(Arc::new(e)))?
            .json::<AutoLoginResponse>()
            .await
            .map_err(|e| CredentialFetchError::JsonDeserializationError(Arc::new(e)))?
            .token;

        let synergia_login_endpoint = format!(
            "/loguj/token/{}/przenies/uczen/widok/centrum_powiadomien",
            auto_login_token.as_inner()
        );

        let resp = self
            .client
            .get(SYNERGIA_URL.join(&synergia_login_endpoint).unwrap())
            .send()
            .await
            .map_err(|e| CredentialFetchError::RequestSendError(Arc::new(e)))?;

        SynergiaCredentials::extract_from_response(&resp)
            .map_err(SynergiaCredentialFetchError::CredentialExtractionError)
    }

    pub async fn refresh(
        &self,
        power_cookie: PowerCookie,
    ) -> Result<SynergiaCredentials, SynergiaCredentialRefreshError> {
        const ENDPOINT: &str = "/refreshToken";
        let mut req = self
            .client
            .get(SYNERGIA_URL.join(ENDPOINT).unwrap())
            .build()
            .map_err(Arc::new)
            .map_err(SynergiaCredentialRefreshError::RefreshRequestConstructionError)?;

        req.append_cookie(power_cookie.clone().into_inner())
            .map_err(SynergiaCredentialRefreshError::CookieInsertError)?;
        let resp = self
            .client
            .execute(req)
            .await
            .map_err(Arc::new)
            .map_err(SynergiaCredentialRefreshError::RefreshRequestSendError)?;

        SynergiaCredentials::extract_from_response_with_powercookie(&resp, power_cookie)
            .map_err(SynergiaCredentialRefreshError::CredentialExtractionError)
    }
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub portal: PortalTokenPair,
    #[allow(unused)]
    pub librus_api: LibrusApiToken,
    pub synergia: SynergiaCredentials,
}

#[derive(Debug)]
pub struct CredentialManager {
    portal: PortalCredentialManager,
    librus_api: LibrusApiCredentialManager,
    synergia: SynergiaCredentialManager,
}

impl CredentialManager {
    pub fn try_new(user_id: SynergiaUserId) -> Result<Self, CredentialManagerError> {
        let client = ClientBuilder::new(
            net::default_client_options()
                .redirect(Policy::none())
                .build()
                .map_err(Arc::new)
                .map_err(CredentialManagerError::ClientConstructionError)?,
        )
        .with(ErrorStatusMiddleware)
        .with(UnauthenticatedErrorMiddleware)
        .build();
        let client = Arc::new(client);
        Ok(Self {
            portal: PortalCredentialManager::new(Arc::clone(&client)),
            librus_api: LibrusApiCredentialManager::new(Arc::clone(&client), user_id),
            synergia: SynergiaCredentialManager::new(client),
        })
    }

    pub(super) async fn new_credentials(
        &self,
        portal_creds: PortalTokenPair,
    ) -> Result<Credentials, CredentialManagerError> {
        match self.librus_api.fetch(&portal_creds.access_token).await {
            Ok(librus_api_creds) => {
                let synergia_creds = self.synergia.fetch(&librus_api_creds).await?;
                Ok(Credentials {
                    portal: portal_creds,
                    librus_api: librus_api_creds,
                    synergia: synergia_creds,
                })
            }
            Err(e) => {
                let LibrusApiCredentialFetchError::GeneralCredentialFetchError(e) = e else {
                    return Err(CredentialManagerError::from(e));
                };

                let CredentialFetchError::RequestSendError(e) = e else {
                    return Err(CredentialManagerError::from(
                        LibrusApiCredentialFetchError::GeneralCredentialFetchError(e),
                    ));
                };

                let reqwest_middleware::Error::Middleware(me) = &*e else {
                    return Err(CredentialManagerError::from(
                        LibrusApiCredentialFetchError::GeneralCredentialFetchError(
                            CredentialFetchError::RequestSendError(e),
                        ),
                    ));
                };

                if me.is::<UnauthenticatedError>() {
                    self.full_refresh_from_token(portal_creds.refresh_token)
                        .await
                } else {
                    Err(CredentialManagerError::from(
                        LibrusApiCredentialFetchError::GeneralCredentialFetchError(
                            CredentialFetchError::RequestSendError(e),
                        ),
                    ))
                }
            }
        }
    }

    async fn full_refresh_from_token(
        &self,
        refresh_token: PortalRefreshToken,
    ) -> Result<Credentials, CredentialManagerError> {
        let portal_creds = self
            .portal
            .fetch(&PortalGrant::RefreshToken(refresh_token))
            .await?;

        let librus_api_creds = self.librus_api.fetch(&portal_creds.access_token).await?;

        let synergia_creds = self.synergia.fetch(&librus_api_creds).await?;

        Ok(Credentials {
            portal: portal_creds,
            librus_api: librus_api_creds,
            synergia: synergia_creds,
        })
    }

    pub async fn full_refresh(
        &self,
        tokens: Credentials,
    ) -> Result<Credentials, CredentialManagerError> {
        self.full_refresh_from_token(tokens.portal.refresh_token)
            .await
    }

    pub fn synergia(&self) -> &SynergiaCredentialManager {
        &self.synergia
    }
}
