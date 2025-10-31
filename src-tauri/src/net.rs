use reqwest::{Client, ClientBuilder, Response};
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

fn default_client_options() -> ClientBuilder {
    const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
    Client::builder()
        .connection_verbose(crate::is_debug())
        .user_agent(USER_AGENT)
}
