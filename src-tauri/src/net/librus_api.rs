use delegate::delegate;
use thiserror::Error;

mod mobile_api;
mod synergia_api;

use crate::net::librus_api::{mobile_api::MobileApi, synergia_api::Messages};
use synergia_api::SynergiaApi;

#[derive(Error, Debug)]
pub enum Error {
    #[error("synergia api error")]
    SynergiaApiError(#[from] synergia_api::Error),
    #[error("librus mobile api error")]
    MobileApiError(#[from] mobile_api::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}

pub struct UnauthenticatedState {
    synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>,
    mobile_api: MobileApi<mobile_api::UnauthenticatedState>,
}

pub struct AuthenticatedState {
    synergia_api: SynergiaApi<synergia_api::AuthenticatedState>,
    mobile_api: MobileApi<mobile_api::UnauthenticatedState>,
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
                mobile_api: MobileApi::try_new()?,
            },
        })
    }

    // this simple only for now
    pub async fn login(self, login: &str, pass: &str) -> Result<LibrusApi<AuthenticatedState>> {
        Ok(LibrusApi::<AuthenticatedState> {
            state: AuthenticatedState {
                synergia_api: self.state.synergia_api.login(login, pass).await?,
                mobile_api: self.state.mobile_api,
            },
        })
    }

    pub async fn mobile_login(self, email: &str, password: &str) -> Result<()> {
        self.state.mobile_api.login(email, password).await?;
        Ok(())
    }
}

impl LibrusApi<AuthenticatedState> {
    delegate! {
        to self.state.synergia_api {
            #[expr(Ok($?))]
            pub async fn messages(&self) -> Result<Messages>;
        }
    }
}
