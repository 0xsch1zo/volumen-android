use std::borrow::Cow;

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{header::ToStrError, redirect::Policy};
use scraper::{Html, Selector};
use thiserror::Error;
use url::Url;

use crate::{
    common::TakeExactlyExt,
    error::StatefulError,
    net::{
        synergia_api::{
            account_selector::{AccountSelector, AccountSelectorConstructionError},
            api::auth::{AuthCode, LoginAttrKinds, LoginAttrs, LoginRequest},
            clients::{UnauthenticatedClient, UnauthenticatedClientConstructionError},
            credential_manager::{
                PortalClientConstructionError, PortalCredentialFetchError, PortalCredentialManager,
            },
            PORTAL_URL,
        },
        SynergiaApi,
    },
    stateful_result,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("state init error")]
    StateInitError(#[from] StateInitError),
    #[error("login attributes request error")]
    LoginAttrRequestError(#[source] reqwest_middleware::Error),
    #[error("failed to read response body to get login attributes")]
    LoginAttrBodyReadError(#[source] reqwest::Error),
    #[error("login attribute not found: {0:?}")]
    LoginAttrNotFound(LoginAttrKinds),
    #[error("auth code request error")]
    AuthCodeRequestError(#[source] reqwest_middleware::Error),
    #[error("invalid auth code location header")]
    InvalidAuthCodeHeader(#[source] ToStrError),
    #[error("auth code url parse error")]
    AuthCodeUrlParseError(#[source] url::ParseError),
    #[error("auth code not found")]
    AuthCodeNotFound,
    #[error("portal cred manager construciton error")]
    PortalCredManagerConstructionError(#[from] PortalClientConstructionError),
    #[error("portal credential fetch error")]
    PortalCredFetchError(#[source] PortalCredentialFetchError),
    #[error("account selector construction error")]
    AccountSelectorConstructionError(#[from] AccountSelectorConstructionError),
}

#[derive(Error, Debug)]
pub enum StateInitError {
    #[error("failed to construct client")]
    ClientConstructionError(#[from] UnauthenticatedClientConstructionError),
}

pub struct UnauthenticatedRedirectPolicy(Policy);

impl UnauthenticatedRedirectPolicy {
    pub fn into_inner(self) -> Policy {
        self.0
    }
}

#[derive(Debug)]
pub struct UnauthenticatedState {
    client: UnauthenticatedClient,
}

impl UnauthenticatedState {
    fn try_new() -> Result<Self, StateInitError> {
        Ok(Self {
            client: UnauthenticatedClient::try_new(
                SynergiaApi::<UnauthenticatedState>::redirect_policy(),
            )?,
        })
    }
}

impl SynergiaApi<UnauthenticatedState> {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            state: UnauthenticatedState::try_new()?,
        })
    }

    fn redirect_policy() -> UnauthenticatedRedirectPolicy {
        UnauthenticatedRedirectPolicy(Policy::custom(|attempt| {
            match Self::extract_auth_code(attempt.url()) {
                Some(_) => attempt.stop(),
                None => attempt.follow(),
            }
        }))
    }

    fn extract_auth_code(url: &Url) -> Option<String> {
        let app_code_url = Url::parse("app://librus").unwrap();
        if app_code_url.scheme() != url.scheme() || app_code_url.domain() != url.domain() {
            return None;
        }

        let query = url.query_pairs().next()?;

        if query.0 == Cow::Borrowed("code") {
            Some(query.1.to_string())
        } else {
            None
        }
    }

    async fn fetch_login_attrs(&self) -> Result<LoginAttrs, Error> {
        fn scrape_attributes(html: &str) -> Result<LoginAttrs, Error> {
            let document = Html::parse_document(html);
            let redirect_to_selector =
                Selector::parse(r#"input[type="hidden"][name="redirectTo"][value]"#).unwrap();
            let redirect_crc_selector =
                Selector::parse(r#"input[type="hidden"][name="redirectCrc"][value]"#).unwrap();
            let token = Selector::parse(r#"input[type="hidden"][name="_token"][value]"#).unwrap();

            let (redirect_to, redirect_crc, token) = [
                (redirect_to_selector, LoginAttrKinds::RedirectTo),
                (redirect_crc_selector, LoginAttrKinds::RedirectCrc),
                (token, LoginAttrKinds::Token),
            ]
            .into_iter()
            .map(|(selector, attr_type)| {
                Ok(document
                    .select(&selector)
                    .take_exactly::<Vec<_>>(1)
                    .inspect_surplus(|_| warn!("many elements matched: {attr_type:?}"))
                    .enough_or(Error::LoginAttrNotFound(attr_type))?[0]
                    .attr("value")
                    .unwrap()
                    .to_owned())
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .collect_tuple()
            .unwrap();

            Ok(LoginAttrs {
                redirect_to,
                redirect_crc,
                token,
            })
        }

        const ATTR_ENDPOINT: &str = "/konto-librus/redirect/dru";

        let html = self
            .state
            .client
            .as_inner()
            .get(PORTAL_URL.join(ATTR_ENDPOINT).unwrap())
            .send()
            .await
            .map_err(Error::LoginAttrRequestError)?
            .text()
            .await
            .map_err(Error::LoginAttrBodyReadError)?;
        scrape_attributes(&html)
    }

    async fn fetch_auth_code(&self, req: &LoginRequest) -> Result<String, Error> {
        const LOGIN_ENDPOINT: &str = "/konto-librus/login/action";
        let auth_code_response = self
            .state
            .client
            .as_inner()
            .post(PORTAL_URL.join(LOGIN_ENDPOINT).unwrap())
            .form(req)
            .send()
            .await
            .map_err(Error::AuthCodeRequestError)?;

        let location_url = auth_code_response
            .headers()
            .get("location")
            .ok_or(Error::AuthCodeNotFound)?
            .to_str()
            .map_err(Error::InvalidAuthCodeHeader)?;
        Ok(Self::extract_auth_code(
            &Url::parse(location_url).map_err(Error::AuthCodeUrlParseError)?,
        )
        .ok_or(Error::AuthCodeNotFound)?)
    }

    pub async fn login(
        self,
        email: String,
        password: String,
    ) -> Result<AccountSelector, StatefulError<Self, Error>> {
        debug!("logging in to librus");
        let attrs = stateful_result! { self => self.fetch_login_attrs().await };

        let auth_code = stateful_result! { self =>
            self
                .fetch_auth_code(&LoginRequest {
                    email,
                    password,
                    attrs,
                })
                .await
                .map(AuthCode::new)
        };

        debug!("successfully logged in");

        let portal_cred_manager = stateful_result! { self  =>
            PortalCredentialManager::try_new()
                .map_err(Error::PortalCredManagerConstructionError)
        };

        let portal_creds = stateful_result! { self =>
            portal_cred_manager.fetch_from_authcode(&auth_code)
                .await
                .map_err(Error::PortalCredFetchError)
        };

        Ok(stateful_result! { self =>
            AccountSelector::try_new(portal_creds)
                .map_err(Error::AccountSelectorConstructionError)
        })
    }
}
