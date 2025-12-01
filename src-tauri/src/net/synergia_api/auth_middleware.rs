use std::sync::Arc;

use log::debug;
use reqwest::{
    cookie::Jar,
    header::{self, InvalidHeaderValue, ToStrError},
    Request, Response, StatusCode,
};
use reqwest_middleware::{Middleware, Next};
use tauri::http::{Extensions, HeaderValue};
use thiserror::Error;

use crate::net::synergia_api::{
    auth_manager::{AuthorizationManager, AuthorizationManagerError},
    internal_types::{PortalAccessToken, SynergiaToken},
    token_management::PickedToken,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest middleware error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("account management error")]
    AuthorizationManagerError(#[from] AuthorizationManagerError),
    #[error("failed to insert auth header")]
    AuthHeaderInsertionFailure(#[from] InvalidHeaderValue),
    #[error("failed to convert cookies to str")]
    CookieConvError(#[source] ToStrError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AuthorizationMiddleware {
    authorization_manager: Arc<AuthorizationManager>,
}

impl AuthorizationMiddleware {
    pub fn new(authorization_manager: Arc<AuthorizationManager>) -> Self {
        Self {
            authorization_manager,
        }
    }

    async fn add_auth_token_on_managed(&self, req: &mut Request) -> Result<()> {
        match self.authorization_manager.managed_token(req.url()).await? {
            Some(PickedToken::PortalAccessToken(portal_token)) => {
                self.add_portal_token(req, portal_token)
            }
            Some(PickedToken::SynergiaToken(synergia_token)) => {
                self.add_synergia_token(req, synergia_token)
            }
            None => Ok(()),
        }
    }

    fn add_portal_token(&self, req: &mut Request, portal_token: PortalAccessToken) -> Result<()> {
        req.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", portal_token.as_inner()))?,
        );
        Ok(())
    }

    fn add_synergia_token(&self, req: &mut Request, synergia_token: SynergiaToken) -> Result<()> {
        let cookies = match req.headers().get(header::COOKIE) {
            Some(c) => format!(
                "{}; oauth_token={}",
                synergia_token.as_inner(),
                c.to_str().map_err(|e| Error::CookieConvError(e))?
            ),
            None => synergia_token.as_inner().to_owned(),
        };
        req.headers_mut()
            .insert(header::COOKIE, HeaderValue::from_str(&cookies)?);
        Ok(())
    }

    async fn handle_unauthorized(
        &self,
        mut req: Request,
        res: Response,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        if self.authorization_manager.is_managed(req.url()).await?
            && res.status() == StatusCode::UNAUTHORIZED
        {
            debug!("refreshing!!!");
            self.authorization_manager
                .refresh() // uses  write lock
                .await?;
            self.add_auth_token_on_managed(&mut req).await?; // uses read lock
            Ok(next.run(req, extensions).await?)
        } else {
            Ok(res)
        }
    }
}

#[async_trait::async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        // https://github.com/TrueLayer/reqwest-middleware/blob/43d31fea66ba23774738d4518da2b4ad40fc346f/reqwest-retry/src/middleware.rs#L146-L149
        // TLDR: this clone should be cheap
        let Some(req_clone) = req.try_clone() else {
            return next.run(req, extensions).await;
        };

        self.add_auth_token_on_managed(&mut req)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;

        let res = next.clone().run(req_clone, extensions).await?;
        let res = self
            .handle_unauthorized(req, res, extensions, next)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;
        Ok(res)
    }
}
