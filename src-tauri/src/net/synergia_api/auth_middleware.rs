use log::debug;
use reqwest::{
    header::{self, InvalidHeaderValue},
    Request, Response, StatusCode,
};
use reqwest_middleware::{Middleware, Next};
use tauri::http::{Extensions, HeaderValue};
use thiserror::Error;

use crate::net::synergia_api::token_management::{
    TokenManager, TokenManagerError, TokenPicker, TokenPickerError,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest middleware error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("token picker error")]
    TokenPickerError(#[from] TokenPickerError),
    #[error("token manager error")]
    TokenManagerError(#[from] TokenManagerError),
    #[error("failed to insert auth header")]
    AuthHeaderInsertionFailure(#[from] InvalidHeaderValue),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AuthorizationMiddleware {
    token_manager: TokenManager,
    token_picker: TokenPicker,
}

impl AuthorizationMiddleware {
    pub fn new(token_manager: TokenManager, token_picker: TokenPicker) -> Self {
        Self {
            token_manager,
            token_picker,
        }
    }

    async fn add_auth_token_on_managed(&self, req: &mut Request) -> Result<()> {
        let tokens = self.token_manager.get().await;
        if let Some(token) = self.token_picker.pick(req.url(), &tokens)? {
            req.headers_mut().insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))?,
            );
        }
        Ok(())
    }

    async fn handle_unauthorized(
        &self,
        mut req: Request,
        res: Response,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let tokens_lock = self.token_manager.get().await;
        let tokens = tokens_lock.clone();
        drop(tokens_lock); // read lock ends here

        let is_managed = self.token_picker.pick(req.url(), &tokens)?.is_some();
        debug!("{is_managed}");
        if is_managed && res.status() == StatusCode::UNAUTHORIZED {
            debug!("refreshing!!!");
            self.token_manager
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
        self.add_auth_token_on_managed(&mut req)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;

        // https://github.com/TrueLayer/reqwest-middleware/blob/43d31fea66ba23774738d4518da2b4ad40fc346f/reqwest-retry/src/middleware.rs#L146-L149
        // TLDR: this clone should be cheap
        let Some(req_clone) = req.try_clone() else {
            return next.run(req, extensions).await;
        };

        let res = next.clone().run(req_clone, extensions).await?;
        let res = self
            .handle_unauthorized(req, res, extensions, next)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;
        Ok(res)
    }
}
