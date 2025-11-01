use std::{borrow::Cow, sync::Arc};

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    cookie::Jar,
    header::{self, InvalidHeaderValue, ToStrError},
    multipart,
    redirect::{Action, Attempt, Policy},
    Client, Url,
};
use scraper::{Html, Selector};
use tauri::http::{HeaderMap, HeaderValue};
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    net::{
        self,
        synergia_api::private_types::{LoginAttrKinds, LoginAttrs, LoginRequest, Tokens},
        ResponseExt,
    },
};

mod private_types;
mod public_types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("url parsing error")]
    UrlParsingError(#[from] url::ParseError),
    #[error("response error")]
    ResponseError(#[from] net::ResponseError),
    #[error("an http header value is invalid")]
    InvalidHeader(#[from] ToStrError),
    #[error("login attribute not found: {0:?}")]
    LoginAttrNotFound(LoginAttrKinds),
    #[error("auth code not found")]
    AuthCodeNotFound,
    #[error("invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}

#[derive(Clone, Debug)]
pub struct UnauthenticatedState {
    client: Client,
    synergia_url: Url,
    portal_url: Url,
    librus_api_url: Url,
}

impl UnauthenticatedState {
    fn try_new() -> Result<Self> {
        Ok(Self {
            client: Self::build_client()?,
            synergia_url: Url::parse("https://synergia.librus.pl").unwrap(),
            portal_url: Url::parse("https://portal.librus.pl").unwrap(),
            librus_api_url: Url::parse("https://api.librus.pl").unwrap(),
        })
    }
    fn build_client() -> Result<Client> {
        Ok(net::default_client_options()
            .redirect(Policy::custom(
                SynergiaApi::<UnauthenticatedState>::redirect_policy,
            ))
            .cookie_store(true)
            .build()?)
    }
}
#[derive(Clone, Debug)]
pub struct AuthenticatedState {
    client: Client,
    synergia_url: Url,
    portal_url: Url,
    librus_api_url: Url,
    refresh_token: String,
}

impl AuthenticatedState {
    fn try_from_tokens(tokens: Tokens) -> Result<Self> {
        Ok(Self {
            client: Self::build_client(&tokens.access_token)?,
            synergia_url: Url::parse("https://synergia.librus.pl").unwrap(),
            portal_url: Url::parse("https://portal.librus.pl").unwrap(),
            librus_api_url: Url::parse("https://api.librus.pl").unwrap(),
            refresh_token: tokens.refresh_token,
        })
    }

    fn build_client(access_token: &str) -> Result<Client> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {access_token}"))?,
        );
        Ok(net::default_client_options()
            .cookie_store(true)
            .default_headers(headers)
            .build()?)
    }
}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

#[derive(Clone, Debug)]
pub struct SynergiaApi<S: ApiState = UnauthenticatedState> {
    state: S,
}

impl SynergiaApi<UnauthenticatedState> {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            state: UnauthenticatedState::try_new()?,
        })
    }

    fn redirect_policy(attempt: Attempt) -> Action {
        match Self::extract_auth_code(attempt.url()) {
            Some(_) => attempt.stop(),
            None => attempt.follow(),
        }
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

    async fn fetch_login_attrs(&self) -> Result<LoginAttrs> {
        fn scrape_attributes(html: &str) -> Result<LoginAttrs> {
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
            .collect::<Result<Vec<_>>>()?
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
            .get(self.state.portal_url.join(ATTR_ENDPOINT).unwrap())
            .send()
            .await?
            .error_on_status()
            .await?
            .text()
            .await?;
        scrape_attributes(&html)
    }

    async fn fetch_auth_code(&self, req: &LoginRequest) -> Result<String> {
        const LOGIN_ENDPOINT: &str = "/konto-librus/login/action";
        let auth_code_response = self
            .state
            .client
            .post(self.state.portal_url.join(LOGIN_ENDPOINT).unwrap())
            .form(req)
            .send()
            .await?
            .error_on_status()
            .await?;

        let location_url = auth_code_response
            .headers()
            .get("location")
            .ok_or(Error::AuthCodeNotFound)?
            .to_str()?;

        Ok(Self::extract_auth_code(&Url::parse(location_url)?).ok_or(Error::AuthCodeNotFound)?)
    }

    async fn fetch_tokens(&self, code: String) -> Result<Tokens> {
        const ACCESS_TOKEN_ENDPOINT: &str = "/oauth2/access_token";
        const CLIENT_ID: &str = "VaItV6oRutdo8fnjJwysnTjVlvaswf52ZqmXsJGP";

        let form = multipart::Form::new()
            .text("grant_type", "authorization_code")
            .text("client_id", CLIENT_ID)
            .text("redirect_uri", "app://librus")
            .text("code", code);

        let tokens = self
            .state
            .client
            .post(self.state.portal_url.join(ACCESS_TOKEN_ENDPOINT).unwrap())
            .multipart(form)
            .send()
            .await?
            .error_on_status()
            .await?
            .json::<Tokens>()
            .await?;

        Ok(tokens)
    }

    pub async fn login(
        self,
        email: &str,
        password: &str,
    ) -> Result<SynergiaApi<AuthenticatedState>> {
        debug!("logging in to librus");
        let attrs = self.fetch_login_attrs().await?;

        let auth_code = self
            .fetch_auth_code(&LoginRequest {
                email: email.to_owned(),
                password: password.to_owned(),
                attrs,
            })
            .await?;

        let tokens = self.fetch_tokens(auth_code).await?;
        debug!("successfully logged in");

        Ok(SynergiaApi::<AuthenticatedState>::try_from_tokens(tokens)?)
    }
}

// We're using the api of the new ui
impl SynergiaApi<AuthenticatedState> {
    fn try_from_tokens(tokens: Tokens) -> Result<Self> {
        Ok(Self {
            state: AuthenticatedState::try_from_tokens(tokens)?,
        })
    }
}
