use std::{borrow::Cow, cell::LazyCell};

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    header::{InvalidHeaderValue, ToStrError},
    redirect::{Action, Attempt, Policy},
    Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use scraper::{Html, Selector};
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    net::{
        self,
        synergia_api::{
            auth_middleware::AuthorizationMiddleware,
            private_types::{LoginAttrKinds, LoginAttrs, LoginRequest, SynergiaUserId},
            token_management::{AuthCode, TokenManager, TokenManagerError, TokenPicker},
        },
        ErrorStatusMiddleware,
    },
};

mod auth_middleware;
mod private_types;
mod public_types;
mod token_management;

const PORTAL_URL: LazyCell<Url> = LazyCell::new(|| Url::parse("https://portal.librus.pl").unwrap());

const SYNERGIA_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://synergia.librus.pl").unwrap());

const LIBRUS_API_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://api.librus.pl").unwrap());

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
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
    #[error("failed to initalize token manager")]
    TokenManagerInitFailed(#[from] TokenManagerError),
    #[error("auth middleware error")]
    AuthMiddlewareError(#[from] auth_middleware::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}

#[derive(Clone, Debug)]
pub struct UnauthenticatedState {
    client: ClientWithMiddleware,
}

impl UnauthenticatedState {
    fn try_new() -> Result<Self> {
        Ok(Self {
            client: Self::build_client()?,
        })
    }

    fn build_client() -> Result<ClientWithMiddleware> {
        let reqwest_client = net::default_client_options()
            .redirect(Policy::custom(
                SynergiaApi::<UnauthenticatedState>::redirect_policy,
            ))
            .cookie_store(true)
            .build()?;

        Ok(ClientBuilder::new(reqwest_client)
            .with(ErrorStatusMiddleware)
            .build())
    }
}

#[derive(Clone, Debug)]
pub struct AuthenticatedState {
    client: ClientWithMiddleware,
}

impl AuthenticatedState {
    fn try_from_token_management(
        token_manager: TokenManager,
        token_picker: TokenPicker,
    ) -> Result<Self> {
        Ok(Self {
            client: Self::build_client(token_manager, token_picker)?,
        })
    }

    fn build_client(
        token_manager: TokenManager,
        token_picker: TokenPicker,
    ) -> Result<ClientWithMiddleware> {
        let reqwest_client = net::default_client_options().cookie_store(true).build()?;

        Ok(ClientBuilder::new(reqwest_client)
            .with(ErrorStatusMiddleware)
            .with(AuthorizationMiddleware::new(token_manager, token_picker))
            .build())
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
            .get(PORTAL_URL.join(ATTR_ENDPOINT).unwrap())
            .send()
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
            .post(PORTAL_URL.join(LOGIN_ENDPOINT).unwrap())
            .form(req)
            .send()
            .await?;

        let location_url = auth_code_response
            .headers()
            .get("location")
            .ok_or(Error::AuthCodeNotFound)?
            .to_str()?;

        Ok(Self::extract_auth_code(&Url::parse(location_url)?).ok_or(Error::AuthCodeNotFound)?)
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

        let token_manager = TokenManager::with_authorized(AuthCode::new(&auth_code)).await?;
        debug!("successfully logged in");

        Ok(SynergiaApi::<AuthenticatedState>::try_from_token_manager(
            token_manager,
        )?)
    }
}

// We're using the api of the new ui
impl SynergiaApi<AuthenticatedState> {
    fn try_from_token_manager(token_manager: TokenManager) -> Result<Self> {
        let picker = TokenPicker::new(SynergiaUserId::new(11111111));
        Ok(Self {
            state: AuthenticatedState::try_from_token_management(token_manager, picker)?,
        })
    }

    pub async fn me(&self) -> Result<String> {
        const ME_ENDPOINT: &str = "/3.0/Me";
        let text = self
            .state
            .client
            .get(LIBRUS_API_URL.join(ME_ENDPOINT).unwrap())
            .header("accept", "application/json")
            .send()
            .await?
            .text()
            .await?;
        Ok(text)
    }
}
