use std::{iter, sync::Arc};

use cookie::Cookie;
use itertools::Itertools;
use log::debug;
use reqwest::{
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
    #[error("failed to parse cookies to attach them to the request")]
    CookieParsingError(#[source] cookie::ParseError),
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
        let token_cookie = Cookie::new("oauth_token", synergia_token.as_inner());
        let cookies = match req.headers().get(header::COOKIE) {
            Some(c) => Cookie::split_parse(c.to_str().map_err(|e| Error::CookieConvError(e))?)
                .collect::<Result<Vec<_>, cookie::ParseError>>()
                .map_err(|e| Error::CookieParsingError(e))?
                .into_iter()
                .chain(iter::once(token_cookie))
                .sorted_unstable_by_key(|c| c.name().to_owned())
                .dedup_by(|a, b| a.name() == b.name())
                .map(|c| c.encoded().stripped().to_string())
                .join("; "),
            None => token_cookie.encoded().stripped().to_string(),
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
            self.authorization_manager.refresh().await?;
            self.add_auth_token_on_managed(&mut req).await?;
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
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        // https://github.com/TrueLayer/reqwest-middleware/blob/43d31fea66ba23774738d4518da2b4ad40fc346f/reqwest-retry/src/middleware.rs#L146-L149
        // TLDR: this clone should be cheap
        let Some(mut req_clone) = req.try_clone() else {
            return next.run(req, extensions).await;
        };

        self.add_auth_token_on_managed(&mut req_clone)
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
