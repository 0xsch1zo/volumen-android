use std::iter;

use cookie::{self, Cookie};
use itertools::Itertools;
use reqwest::{
    header::{self, InvalidHeaderValue, ToStrError},
    Client, ClientBuilder, Request, Response,
};
use reqwest_middleware::{Middleware, Next};
use tauri::http::{Extensions, HeaderMap, HeaderValue};
use thiserror::Error;

pub mod synergia_api;

pub use synergia_api::{Error as SynergiaApiError, SynergiaApi};

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("a response returned an error status code, with body: {0}")]
    ErrorWithBody(String, #[source] reqwest::Error),
    #[error("a response returned an error status code, couldn't read the body")]
    ErrorNoBody(#[from] reqwest::Error),
}

trait ResponseExt: Sized {
    async fn error_on_status(self) -> Result<Self, ResponseError>;
}

impl ResponseExt for Response {
    async fn error_on_status(self) -> Result<Self, ResponseError> {
        if self.status().is_client_error() || self.status().is_server_error() {
            let error = self.error_for_status_ref().unwrap_err();
            let body = self.text().await?;
            return Err(ResponseError::ErrorWithBody(body, error));
        } else {
            Ok(self)
        }
    }
}

struct ErrorStatusMiddleware;

#[async_trait::async_trait]
impl Middleware for ErrorStatusMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        let response = next.run(req, extensions).await?;
        Ok(response
            .error_on_status()
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?)
    }
}

/*trait UrlCompareExt {
    fn is_same_base(&self, other: &Url) -> bool;

    fn starts_with(&self, other: &Url) -> bool;
}

impl UrlCompareExt for Url {
    fn is_same_base(&self, other: &Url) -> bool {
        self.has_host() && self.host() == other.host() && self.scheme() == other.scheme()
    }

    fn starts_with(&self, other: &Url) -> bool {
        if !self.is_same_base(other) {
            return false;
        }

        return self.path().starts_with(other.path());
    }
}*/

fn default_client_options() -> ClientBuilder {
    const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
    Client::builder()
        .connection_verbose(crate::is_debug())
        .user_agent(USER_AGENT)
}

#[derive(Error, Debug)]
pub enum RequestCookieExtError {
    #[error("failed to convert cookie header to &str")]
    CookieHeaderToStrError(#[source] ToStrError),
    #[error("failed parse to parse cookies from the header")]
    CookieParseError(#[source] cookie::ParseError),
    #[error("failed to insert new cookie header")]
    CookieHeaderInsertionError(#[source] InvalidHeaderValue),
}

// maybe expand to a new type if needed
trait RequestCookieExt {
    fn append_cookie(&mut self, cookie: Cookie) -> Result<(), RequestCookieExtError>;

    fn contains_cookie(&self, cookie_name: &str) -> Result<bool, RequestCookieExtError>;
}

impl RequestCookieExt for Request {
    fn append_cookie(&mut self, cookie: Cookie) -> Result<(), RequestCookieExtError> {
        let cookies = match self.headers().get(header::COOKIE) {
            Some(c) => Cookie::split_parse_encoded(
                c.to_str()
                    .map_err(RequestCookieExtError::CookieHeaderToStrError)?,
            )
            .collect::<Result<Vec<_>, cookie::ParseError>>()
            .map_err(RequestCookieExtError::CookieParseError)?
            .into_iter()
            .chain(iter::once(cookie))
            .map(|c| c.encoded().stripped().to_string())
            .join("; "),
            None => cookie.encoded().stripped().to_string(),
        };

        self.headers_mut().insert(
            header::COOKIE,
            HeaderValue::from_str(&cookies)
                .map_err(RequestCookieExtError::CookieHeaderInsertionError)?,
        );

        Ok(())
    }

    fn contains_cookie(&self, cookie_name: &str) -> Result<bool, RequestCookieExtError> {
        let cookie_headers = self
            .headers()
            .get_all(header::COOKIE)
            .into_iter()
            .map(HeaderValue::to_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(RequestCookieExtError::CookieHeaderToStrError)?;

        let cookies = cookie_headers
            .into_iter()
            .map(Cookie::split_parse_encoded)
            .map(|cookies| cookies.collect::<Result<Vec<_>, cookie::ParseError>>())
            .collect::<Result<Vec<_>, cookie::ParseError>>()
            .map_err(RequestCookieExtError::CookieParseError)?
            .into_iter()
            .flatten();

        Ok(cookies.into_iter().any(|c| c.name() == cookie_name))
    }
}

#[derive(Error, Debug)]
enum ResponseCookieExtError {
    #[error("failed to convert cookie header to &str")]
    CookieHeaderToStrError(#[source] ToStrError),
    #[error("failed parse to parse cookies from the header")]
    CookieParseError(#[source] cookie::ParseError),
}

trait ResponseCookieExt {
    fn extract_cookie(&self, name: &str) -> Result<Option<Cookie>, ResponseCookieExtError>;
}

impl ResponseCookieExt for Response {
    fn extract_cookie(&self, name: &str) -> Result<Option<Cookie>, ResponseCookieExtError> {
        let cookie_headers = self
            .headers()
            .get_all(header::COOKIE)
            .into_iter()
            .map(HeaderValue::to_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(ResponseCookieExtError::CookieHeaderToStrError)?;

        let cookies = cookie_headers
            .into_iter()
            .map(Cookie::split_parse_encoded)
            .map(|cookies| cookies.collect::<Result<Vec<_>, cookie::ParseError>>())
            .collect::<Result<Vec<_>, cookie::ParseError>>()
            .map_err(ResponseCookieExtError::CookieParseError)?
            .into_iter()
            .flatten();

        Ok(cookies.into_iter().rfind(|c| c.name() == name))
    }
}
