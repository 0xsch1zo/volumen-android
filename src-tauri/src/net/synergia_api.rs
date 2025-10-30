use std::sync::Arc;

use log::debug;
use reqwest::{
    cookie::{CookieStore, Jar},
    header::ToStrError,
    Client, Url,
};
use thiserror::Error;

use crate::net::{
    self,
    synergia_api::private_types::{LoginRequest, LoginResponse},
    ResponseExt,
};

mod private_types;
mod public_types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("url parsing error")]
    UrlParsingError(#[from] url::ParseError),
    #[error("response error")]
    ResponseError(#[from] net::ResponseError),
    #[error("an http header value is invalid")]
    InvalidHeader(#[from] ToStrError),
    #[error("failed to acquire power cookies")]
    FailedToGetPowerCookies,
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}

pub struct UnauthenticatedState {
    cookie_store: Arc<Jar>,
    client: Client,
    synergia_url: Url,
    librus_api_url: Url,
}

pub struct AuthenticatedState {
    cookie_store: Arc<Jar>,
    client: Client,
    synergia_url: Url,
    librus_api_url: Url,
}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

pub struct SynergiaApi<S: ApiState = UnauthenticatedState> {
    state: S,
}

impl SynergiaApi<UnauthenticatedState> {
    fn build_client(cookie_store: &Arc<Jar>) -> Result<Client> {
        Ok(net::default_client_options()
            .cookie_provider(Arc::clone(cookie_store))
            .build()?)
    }

    pub async fn with_authorized() -> Result<Self> {
        let cookie_store = Arc::new(Jar::default());
        let api = Self {
            state: UnauthenticatedState {
                client: Self::build_client(&cookie_store)?,
                cookie_store,
                synergia_url: Url::parse("https://synergia.librus.pl").unwrap(),
                librus_api_url: Url::parse("https://api.librus.pl").unwrap(),
            },
        };
        api.acquire_power_cookies().await?;
        Ok(api)
    }

    /// This function acquires what I'm referring to as power cookies, which are weired cookies that
    /// librus sends to you on every request to their api, and doesn't proceed to handling any
    /// request unless you have them
    async fn acquire_power_cookies(&self) -> Result<()> {
        fn have_power_cookies(cookie_store: &Jar, url: &Url) -> Result<bool> {
            const POWER_COOKIES_NAME: [&str; 2] = ["DZIENNIKSID", "SDZIENNIKSID"];
            let Some(cookies) = cookie_store.cookies(url) else {
                return Ok(false);
            };

            let cookies = cookies.to_str()?.split(";");
            let have_power_cookies = POWER_COOKIES_NAME
                .iter()
                .all(|power_cookie| cookies.clone().any(|cookie| cookie.contains(power_cookie)));
            Ok(have_power_cookies)
        }

        match have_power_cookies(&self.state.cookie_store, &self.state.librus_api_url) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(e) => return Err(e),
        };

        debug!("acquiring power cookies");

        const LOGIN_ENDPOINT: &str = "/loguj/portalRodzina";
        self.state
            .client
            .get(self.state.synergia_url.join(LOGIN_ENDPOINT).unwrap())
            .send()
            .await?;

        match have_power_cookies(&self.state.cookie_store, &self.state.librus_api_url) {
            Ok(true) => {
                debug!("successfully acquired power cookies");
                Ok(())
            }
            Ok(false) => Err(Error::FailedToGetPowerCookies),
            Err(e) => Err(e),
        }
    }

    pub async fn login(self, login: &str, pass: &str) -> Result<SynergiaApi<AuthenticatedState>> {
        const AUTH_ENPOINT: &str = "/OAuth/Authorization?client_id=46"; // why 46 you may ask, ...
                                                                        // I don't know
        debug!("logging in to synergia");
        let resp = self
            .state
            .client
            .post(self.state.librus_api_url.join(AUTH_ENPOINT).unwrap())
            .form(&LoginRequest {
                action: "login".to_owned(),
                login: login.to_owned(),
                pass: pass.to_owned(),
            })
            .send()
            .await?
            .error_on_status()
            .await?
            .json::<LoginResponse>()
            .await?;

        self.state
            .client
            .get(self.state.librus_api_url.join(&resp.go_to)?)
            .send()
            .await?;

        debug!("successfully logged in");
        Ok(SynergiaApi {
            state: AuthenticatedState {
                client: self.state.client,
                cookie_store: self.state.cookie_store,
                librus_api_url: self.state.librus_api_url,
                synergia_url: self.state.synergia_url,
            },
        })
    }
}

// We're using the api of the new ui
/*impl SynergiaApi<AuthenticatedState> {
    pub async fn messages(&self) -> Result<Messages> {
        const MESSAGES_ENDPOINT: &str = "/wiadomosci";
        debug!("fetching messages");
        let html = self
            .state
            .client
            .get(self.state.synergia_url.join(MESSAGES_ENDPOINT).unwrap())
            .send()
            .await?
            .error_on_status()
            .await?
            .text()
            .await?;
        debug!("successfully fetched messages");

        todo!()
    }
}*/
