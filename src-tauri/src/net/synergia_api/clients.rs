use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use cookie::Cookie;
use log::debug;
use reqwest::{Request, Response, StatusCode};
use reqwest_cookie_store::{CookieStore, CookieStoreRwLock};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use tauri::http::{Extensions, HeaderValue};
use thiserror::Error;

use crate::net::{
    self,
    synergia_api::{
        api::auth::PowerCookie,
        auth_manager::{self, AuthorizationManager, MessagesAuthManager, MessagesManagerError},
        UnauthenticatedRedirectPolicy,
    },
    ErrorStatusMiddleware, RequestCookieExt, RequestCookieExtError,
};

#[derive(Error, Debug)]
enum MiddlewareError {
    #[error("next failed: an error occured in the middleware chain")]
    MiddlewareChainFailure(#[source] reqwest_middleware::Error),
    #[error("failed to acquire synergia auth token")]
    FailedTokenAcquistion(#[source] auth_manager::ManagerError),
    #[error("token refresh failure")]
    TokenRefreshError(#[source] auth_manager::ManagerError),
    #[error("failed to append auth token to cookie header")]
    AuthCookieAppendageError(#[source] RequestCookieExtError),
}

struct AuthorizationMiddleware {
    auth_manager: Arc<AuthorizationManager>,
}

impl AuthorizationMiddleware {
    fn new(auth_manager: Arc<AuthorizationManager>) -> Self {
        Self { auth_manager }
    }

    // TODO: Maybe refactor to use cookies
    async fn authenticate(&self, req: &mut Request) -> Result<(), MiddlewareError> {
        let token = self
            .auth_manager
            .authenciation()
            .await
            .map_err(MiddlewareError::FailedTokenAcquistion)?;
        let token_cookie = Cookie::new("oauth_token", token.as_inner());
        req.append_cookie(token_cookie)
            .map_err(MiddlewareError::AuthCookieAppendageError)?;
        Ok(())
    }

    async fn handle_unauthorized(
        &self,
        mut req: Request,
        res: Response,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, MiddlewareError> {
        //if res.status() == StatusCode::UNAUTHORIZED {
        debug!("refreshing!!!");
        self.auth_manager
            .refresh()
            .await
            .map_err(MiddlewareError::TokenRefreshError)?;
        self.authenticate(&mut req).await?;
        Ok(next
            .run(req, extensions)
            .await
            .map_err(MiddlewareError::MiddlewareChainFailure)?)
        /*} else {
            Ok(res)
        }*/
    }
}

#[async_trait::async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        // https://github.com/TrueLayer/reqwest-middleware/blob/43d31fea66ba23774738d4518da2b4ad40fc346f/reqwest-retry/src/middleware.rs#L146-L149
        // TLDR: this clone should be cheap
        let Some(mut req_clone) = req.try_clone() else {
            return next.run(req, extensions).await;
        };

        self.authenticate(&mut req_clone)
            .await
            .map_err(reqwest_middleware::Error::middleware)?;

        let res = next.clone().run(req_clone, extensions).await?;
        let res = self
            .handle_unauthorized(req, res, extensions, next)
            .await
            .map_err(reqwest_middleware::Error::middleware)?;
        Ok(res)
    }
}

#[derive(Error, Debug)]
pub enum MessagesMiddlewareError {
    #[error("failed to construct MessagesAuthManager")]
    MessagesAuthManagerConstructionError(#[source] MessagesManagerError),
    #[error("failed to authorize access to the messages module")]
    AuthorizatioError(#[source] MessagesManagerError),
    #[error("failed to fetch power cookie")]
    PowerCookieFetchError(#[source] MessagesManagerError),
    #[error("failed to insert power cookie")]
    PowerCookieInsertError(#[source] cookie_store::CookieError),
    #[error("failed to check if request contains power cookie")]
    RequestAuthCheckError(#[source] RequestCookieExtError),
    #[error("failed to append power cookie to request")]
    PowerCookieAppendageError(#[source] RequestCookieExtError),
}

struct MessagesAuthorizationMiddleware {
    messages_auth_manager: MessagesAuthManager,
    cookie_store: Arc<CookieStoreRwLock>,
}

impl MessagesAuthorizationMiddleware {
    fn try_new(
        cookie_store: Arc<CookieStoreRwLock>,
        auth_manager: Arc<AuthorizationManager>,
    ) -> Result<Self, MessagesMiddlewareError> {
        MessagesAuthManager::try_new(auth_manager)
            .map(|messages_auth_manager| Self {
                cookie_store,
                messages_auth_manager,
            })
            .map_err(MessagesMiddlewareError::MessagesAuthManagerConstructionError)
    }

    async fn authenticate(&self, req: &mut Request) -> Result<(), MessagesMiddlewareError> {
        debug!("requesting authorization");
        let power_cookie = self
            .cookie_store
            .read()
            .unwrap()
            .get(PowerCookie::DOMAIN, PowerCookie::PATH, PowerCookie::NAME)
            .map(ToOwned::to_owned)
            .map(cookie_store::Cookie::into_owned)
            .map(PowerCookie::new);

        let resp = self
            .messages_auth_manager
            .request_authoriztaion(power_cookie)
            .await
            .map_err(MessagesMiddlewareError::AuthorizatioError)?;
        debug!("authorization response: {resp:?}");
        /*let power_cookie = self
            .cookie_store
            .read()
            .unwrap()
            .get(PowerCookie::DOMAIN, PowerCookie::PATH, PowerCookie::NAME)
            .map(ToOwned::to_owned)
            .map(cookie_store::Cookie::into_owned);

        let power_cookie_is_attached = req
            .contains_cookie(PowerCookie::NAME)
            .map_err(MessagesMiddlewareError::RequestAuthCheckError)?;

        match (power_cookie, power_cookie_is_attached) {
            (Some(_), true) => return Ok(()),
            (Some(power_cookie), false) => req
                .append_cookie(power_cookie.into())
                .map_err(MessagesMiddlewareError::PowerCookieAppendageError)?,
            (None, _) => {
                let power_cookie = self
                    .messages_auth_manager
                    .fetch_power_cookie()
                    .await
                    .map_err(MessagesMiddlewareError::PowerCookieFetchError)?
                    .into_inner();
                self.cookie_store
                    .write()
                    .unwrap()
                    .insert(power_cookie.clone(), &PowerCookie::URL)
                    .map_err(MessagesMiddlewareError::PowerCookieInsertError)?;
                debug!("{:?}", self.cookie_store.read().unwrap());

                debug!("inserting cookie");
                req.append_cookie(power_cookie.into())
                    .map_err(MessagesMiddlewareError::PowerCookieAppendageError)?;
            }
        }*/

        Ok(())
    }
}

#[async_trait::async_trait]
impl Middleware for MessagesAuthorizationMiddleware {
    // for now we don't do retries here
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        self.authenticate(&mut req)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;
        debug!("authetnicated");

        next.run(req, extensions).await
    }
}

#[derive(Error, Debug)]
pub enum ClientConstructionError {
    #[error("failed to construct reqwest client")]
    ReqwestClientFailure(#[from] reqwest::Error),
    #[error("failed to construct reqwest middleware client")]
    ReqwestMiddlewareClientFailure(#[from] reqwest_middleware::Error),
    #[error("messages middleware construction failure")]
    MessagesMiddlewareConstructionError(#[source] MessagesMiddlewareError),
}

#[derive(Debug)]
pub struct UnauthenticatedClient(ClientWithMiddleware);

impl UnauthenticatedClient {
    pub fn try_new(
        cookie_store: Arc<CookieStoreRwLock>,
        redirect_policy: UnauthenticatedRedirectPolicy,
    ) -> Result<Self, ClientConstructionError> {
        let client = net::default_client_options()
            .redirect(redirect_policy.into_inner())
            .cookie_provider(cookie_store)
            .build()?;

        Ok(Self(
            ClientBuilder::new(client)
                .with(ErrorStatusMiddleware)
                .build(),
        ))
    }

    pub fn as_inner(&self) -> &ClientWithMiddleware {
        &self.0
    }
}

#[derive(Debug)]
pub struct AuthenticatedClient(ClientWithMiddleware);

impl AuthenticatedClient {
    pub fn try_new(
        authorization_manager: Arc<AuthorizationManager>,
    ) -> Result<Self, ClientConstructionError> {
        let cookie_store = Arc::new(CookieStoreRwLock::new(CookieStore::new()));
        let client = net::default_client_options()
            .cookie_provider(Arc::clone(&cookie_store))
            .build()?;

        Ok(Self(
            ClientBuilder::new(client)
                .with(ErrorStatusMiddleware)
                .with(AuthorizationMiddleware::new(Arc::clone(
                    &authorization_manager,
                )))
                /*.with(
                    MessagesAuthorizationMiddleware::try_new(cookie_store, authorization_manager)
                        .map_err(ClientConstructionError::MessagesMiddlewareConstructionError)?,
                )*/
                .build(),
        ))
    }

    pub fn as_inner(&self) -> &ClientWithMiddleware {
        &self.0
    }
}
