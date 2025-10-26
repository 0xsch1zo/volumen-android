use thiserror::Error;

use crate::net::{
    scraper,
    synergia_api::{self, SynergiaApi},
};

pub use scraper::{Message, Messages};

#[derive(Error, Debug)]
pub enum Error {
    #[error("synergia api error")]
    SynergiaApiError(#[from] synergia_api::Error),
    #[error("scraping error")]
    ScrapingError(#[from] scraper::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}

pub struct UnauthenticatedState {
    synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>,
}

pub struct AuthenticatedState {
    synergia_api: SynergiaApi<synergia_api::AuthenticatedState>,
}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

pub struct LibrusApi<S: ApiState = UnauthenticatedState> {
    state: S,
}

impl LibrusApi<UnauthenticatedState> {
    pub async fn with_authorized() -> Result<Self> {
        Ok(Self {
            state: UnauthenticatedState {
                synergia_api: SynergiaApi::with_authorized().await?,
            },
        })
    }

    // this simple only for now
    pub async fn login(self, login: &str, pass: &str) -> Result<LibrusApi<AuthenticatedState>> {
        Ok(LibrusApi::<AuthenticatedState> {
            state: AuthenticatedState {
                synergia_api: self.state.synergia_api.login(login, pass).await?,
            },
        })
    }
}

impl LibrusApi<AuthenticatedState> {
    pub async fn messages(&self) -> Result<Messages> {
        let messages_html = self.state.synergia_api.messages().await?;
        Ok(scraper::scrape_messages(&messages_html)?)
    }
}
