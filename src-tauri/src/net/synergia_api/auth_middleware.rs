use std::iter;

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

use crate::net::synergia_api::auth_manager::{self, AuthorizationManager};

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest middleware error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("failed to insert auth header")]
    AuthHeaderInsertionFailure(#[from] InvalidHeaderValue),
    #[error("failed to convert cookies to str")]
    CookieConvError(#[source] ToStrError),
    #[error("failed to parse cookies to attach them to the request")]
    CookieParsingError(#[source] cookie::ParseError),
    #[error("failed to acquire synergia auth token")]
    FailedTokenAcquistion(#[source] auth_manager::Error),
    #[error("token refresh failure")]
    TokenRefreshError(#[source] auth_manager::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AuthorizationMiddleware {
    auth_manager: AuthorizationManager,
}

impl AuthorizationMiddleware {
    pub fn new(auth_manager: AuthorizationManager) -> Self {
        Self { auth_manager }
    }

    async fn authenticate(&self, req: &mut Request) -> Result<()> {
        let token = self
            .auth_manager
            .authenciation()
            .await
            .map_err(Error::FailedTokenAcquistion)?;
        let token_cookie = Cookie::new("oauth_token", token.as_inner());
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
        if res.status() == StatusCode::UNAUTHORIZED {
            debug!("refreshing!!!");
            self.auth_manager
                .refresh()
                .await
                .map_err(Error::TokenRefreshError)?;
            self.authenticate(&mut req).await?;
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

        self.authenticate(&mut req_clone)
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
