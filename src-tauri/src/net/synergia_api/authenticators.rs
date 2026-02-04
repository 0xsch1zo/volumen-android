use std::sync::Arc;

use cookie::Cookie;
use log::{debug, error};
use reqwest::{redirect::Policy, Request, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use tauri::http::Extensions;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::{
    net::{
        self,
        synergia_api::{
            api::auth::{PortalTokenPair, SynergiaToken, SynergiaUserId},
            credential_manager::{
                CredentialManager, CredentialManagerError, Credentials,
                SynergiaCredentialRefreshError,
            },
            AuthenticatedSynergiaEndpoints, MessagesEndpoints,
        },
        RequestCookieExt, RequestCookieExtError,
    },
    sync::SingleParallelFlight,
};

#[derive(Error, Clone, Debug)]
pub enum MainAuthenticatorError {
    #[error("credential manager construction error")]
    CredentialManagerConstructionError(#[source] CredentialManagerError),
    #[error("failed to fetch new credentials")]
    CredententialFetchError(#[source] CredentialManagerError),
    #[error("failed to add authentication to request")]
    RequestAuthenticationError(#[source] RequestCookieExtError),
    #[error(
        "fatal credential refresh error\nPortal and synergia refresh failure\n Full refresh CredentialManagerError: {:?}", .0
    )]
    FatalCredentialRefreshError(
        CredentialManagerError,
        #[source] SynergiaCredentialRefreshError,
    ),
    #[error("request send error")]
    RequestSendError(#[source] Arc<reqwest_middleware::Error>),
}

pub struct MainAuthenticator {
    refresh_worker: SingleParallelFlight<Result<(), MainAuthenticatorError>>,
    credentials: RwLock<Option<Credentials>>,
    credential_manager: CredentialManager,
}

impl MainAuthenticator {
    // TODO: maybe add stateful error in here
    pub async fn init(
        user_id: SynergiaUserId,
        portal_creds: PortalTokenPair,
    ) -> Result<Self, MainAuthenticatorError> {
        let credential_manager = CredentialManager::try_new(user_id)
            .map_err(MainAuthenticatorError::CredentialManagerConstructionError)?;
        let credentials = credential_manager
            .new_credentials(portal_creds)
            .await
            .map_err(MainAuthenticatorError::CredententialFetchError)?;
        let s = Self {
            refresh_worker: SingleParallelFlight::new(),
            credentials: RwLock::new(Some(credentials)),
            credential_manager,
        };
        s.refresh().await?;
        Ok(s)
    }

    async fn refresh(&self) -> Result<(), MainAuthenticatorError> {
        self.refresh_worker
            .work(async || {
                let mut cred_guard = self.credentials.write().await;
                let creds = cred_guard.take().unwrap();

                let synergia_creds = self
                    .credential_manager
                    .synergia()
                    .refresh(creds.synergia.power_cookie.clone())
                    .await;

                match synergia_creds {
                    Ok(synergia_creds) => {
                        *cred_guard = Some(Credentials {
                            synergia: synergia_creds,
                            ..creds
                        });
                    }
                    Err(synergia_err) => {
                        error!("synergia refresh failed: {synergia_err:?}");
                        let creds = self.credential_manager.refresh(creds).await.map_err(|e| {
                            MainAuthenticatorError::FatalCredentialRefreshError(e, synergia_err)
                        })?;
                        *cred_guard = Some(creds);
                    }
                };

                Ok(())
            })
            .await?;
        Ok(())
    }

    async fn authenticate(&self, req: &mut Request) -> Result<(), MainAuthenticatorError> {
        let cred_guard = self.credentials.read().await;
        let token_cookie = cred_guard
            .as_ref()
            .unwrap()
            .synergia
            .token
            .as_inner()
            .to_owned();
        req.append_cookie(token_cookie)
            .map_err(MainAuthenticatorError::RequestAuthenticationError)?;
        Ok(())
    }

    async fn on_unauthorized(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, MainAuthenticatorError> {
        debug!("refreshing!!!");
        self.refresh().await?;
        self.authenticate(&mut req).await?;
        Ok(next
            .run(req, extensions)
            .await
            .map_err(Arc::new)
            .map_err(MainAuthenticatorError::RequestSendError)?)
    }
}

#[async_trait::async_trait]
impl Middleware for MainAuthenticator {
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
        match res.status() {
            StatusCode::UNAUTHORIZED => self
                .on_unauthorized(req, extensions, next)
                .await
                .map_err(reqwest_middleware::Error::middleware),
            _ => Ok(res),
        }
    }
}

#[derive(Error, Debug)]
pub enum MessagesAuthenticatorError {
    #[error("failed to construct client")]
    ClientInitError(#[source] reqwest::Error),
    #[error("failed to insert authorization cookie")]
    AuthCookieInsertError(#[source] RequestCookieExtError),
}

pub struct MessagesAuthenticator {
    main_authenticator: Arc<MainAuthenticator>,
    client: ClientWithMiddleware,
}

impl MessagesAuthenticator {
    pub async fn init(
        main_authenticator: Arc<MainAuthenticator>,
    ) -> Result<Self, MessagesAuthenticatorError> {
        let client = ClientBuilder::new(
            net::default_client_options()
                .redirect(Self::redirect_policy())
                .build()
                .map_err(MessagesAuthenticatorError::ClientInitError)?,
        )
        .build();

        let authenticator = Self {
            main_authenticator,
            client,
        };
        authenticator.authorize().await?;
        Ok(authenticator)
    }

    fn redirect_policy() -> Policy {
        Policy::custom(|attempt| {
            debug!("following redirect: {}", attempt.url());
            attempt.follow()
        })
    }

    async fn authorize(&self) -> Result<(), MessagesAuthenticatorError> {
        let cred_gaurd = self.main_authenticator.credentials.read().await;
        let creds = cred_gaurd.as_ref().unwrap();

        let mut req = self
            .client
            .get(AuthenticatedSynergiaEndpoints::Messages(MessagesEndpoints::Authorization).url())
            .build()
            .map_err(MessagesAuthenticatorError::ClientInitError)?;

        req.append_cookie(creds.synergia.token.as_inner().to_owned())
            .map_err(MessagesAuthenticatorError::AuthCookieInsertError)?;

        req.append_cookie(creds.synergia.power_cookie.as_inner().to_owned())
            .map_err(MessagesAuthenticatorError::AuthCookieInsertError)?;

        let resp = self.client.execute(req);
        // TODO
        Ok(())
    }

    async fn authenticate(&self, req: &mut Request) -> Result<(), MainAuthenticatorError> {
        let cred_guard = self.main_authenticator.credentials.read().await;
        let power_cookie = cred_guard
            .as_ref()
            .unwrap()
            .synergia
            .power_cookie
            .as_inner()
            .to_owned();
        req.append_cookie(power_cookie)
            .map_err(MainAuthenticatorError::RequestAuthenticationError)?;
        Ok(())
    }

    async fn on_unauthorized(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, MainAuthenticatorError> {
        self.main_authenticator.refresh().await?;
        self.authenticate(&mut req).await?;
        Ok(next
            .run(req, extensions)
            .await
            .map_err(Arc::new)
            .map_err(MainAuthenticatorError::RequestSendError)?)
    }
}

#[async_trait::async_trait]
impl Middleware for MessagesAuthenticator {
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
        match res.status() {
            StatusCode::UNAUTHORIZED => self
                .on_unauthorized(req, extensions, next)
                .await
                .map_err(reqwest_middleware::Error::middleware),
            _ => Ok(res),
        }
    }
}
