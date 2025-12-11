use std::{borrow::Cow, cell::LazyCell, sync::Arc};

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    header::{InvalidHeaderValue, ToStrError},
    redirect::{Action, Attempt, Policy},
    Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use scraper::{Html, Selector};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    error,
    net::{
        self,
        synergia_api::{
            account_selector::AccountSelector,
            auth_manager::AuthorizationManager,
            auth_middleware::AuthorizationMiddleware,
            internal_types::{LoginAttrKinds, LoginAttrs, LoginRequest, RawComment},
            token_management::{AuthCode, TokenManager, TokenManagerError},
        },
        ErrorStatusMiddleware,
    },
    repositories::{
        grades::{Categories, Comment, CommentId, ShallowGrades},
        subjects::Subjects,
        users::Users,
    },
    stateful_result,
};

pub mod account_selector;
mod auth_manager;
mod auth_middleware;
mod internal_types;
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

type StatefulError<S, E = Error> = error::StatefulError<S, E>;
pub trait ApiState {}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct AuthenticatedState {
    client: ClientWithMiddleware,
}

impl AuthenticatedState {
    fn try_from_auth_manager(
        authorization_manager: AuthorizationManager,
    ) -> Result<Self, StatefulError<AuthorizationManager>> {
        Ok(Self {
            client: Self::build_client(authorization_manager)?,
        })
    }

    fn build_client(
        authorization_manager: AuthorizationManager,
    ) -> Result<ClientWithMiddleware, StatefulError<AuthorizationManager>> {
        let reqwest_client = stateful_result! { authorization_manager =>
            net::default_client_options().cookie_store(true).build()
        };

        Ok(ClientBuilder::new(reqwest_client)
            .with(ErrorStatusMiddleware)
            .with(AuthorizationMiddleware::new(Arc::new(
                authorization_manager,
            )))
            .build())
    }
}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

#[derive(Debug)]
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
    ) -> Result<AccountSelector, StatefulError<Self>> {
        debug!("logging in to librus");
        let attrs = stateful_result! { self => self.fetch_login_attrs().await };

        let auth_code = stateful_result! { self =>
            self
                .fetch_auth_code(&LoginRequest {
                    email: email.to_owned(),
                    password: password.to_owned(),
                    attrs,
                })
                .await
        };

        let token_manager = stateful_result! { self =>
            TokenManager::with_authorized(AuthCode::new(&auth_code)).await
        };
        debug!("successfully logged in");

        Ok(AccountSelector::new(token_manager))
    }
}

#[derive(Debug)]
enum AuthenticatedSynergiaEndpoints {
    Users,
    Grades,
    Subjects,
    Categories,
    Comments(CommentId),
}

impl AuthenticatedSynergiaEndpoints {
    fn path(&self) -> String {
        match self {
            AuthenticatedSynergiaEndpoints::Users => "/gateway/api/2.0/Users".to_owned(),
            AuthenticatedSynergiaEndpoints::Grades => "/gateway/api/2.0/Grades".to_owned(),
            AuthenticatedSynergiaEndpoints::Subjects => "/gateway/api/2.0/Subjects".to_owned(),
            AuthenticatedSynergiaEndpoints::Categories => {
                "/gateway/api/2.0/Grades/Categories".to_owned()
            }
            AuthenticatedSynergiaEndpoints::Comments(id) => {
                format!("/gateway/api/2.0/Grades/Comments/{}", id.inner())
            }
        }
    }
}

// We're using the api of the new ui
impl SynergiaApi<AuthenticatedState> {
    fn try_from_auth_manager(
        authorization_manager: AuthorizationManager,
    ) -> Result<Self, StatefulError<AuthorizationManager>> {
        Ok(Self {
            state: AuthenticatedState::try_from_auth_manager(authorization_manager)?,
        })
    }

    async fn fetch_synergia_endpoint<T: DeserializeOwned>(
        &self,
        endpoint: AuthenticatedSynergiaEndpoints,
    ) -> Result<T> {
        debug!("fetching {endpoint:?}");
        let resource = self
            .state
            .client
            .get(SYNERGIA_URL.join(&endpoint.path()).unwrap())
            .send()
            .await?
            .json::<T>()
            .await?;
        debug!("fetched {endpoint:?} succesfully");
        Ok(resource)
    }

    // TODO: actually properly parse the data or something
    pub async fn me(&self) -> Result<String> {
        const ME_ENDPOINT: &str = "/3.0/Me";
        debug!("fetching \"me\" info");
        let text = self
            .state
            .client
            .get(LIBRUS_API_URL.join(ME_ENDPOINT).unwrap())
            .header("accept", "application/json")
            .send()
            .await?
            .text()
            .await?;
        debug!("successfully fetched \"me\" info");
        Ok(text)
    }

    // fetch_synergia_endpoint isn't exposed because we don't want to let the user be able to
    // choose a wrong type for the output on accident
    pub async fn users(&self) -> Result<Users> {
        Ok(self
            .fetch_synergia_endpoint(AuthenticatedSynergiaEndpoints::Users)
            .await?)
    }

    pub async fn grades(&self) -> Result<ShallowGrades> {
        Ok(self
            .fetch_synergia_endpoint(AuthenticatedSynergiaEndpoints::Grades)
            .await?)
    }

    pub async fn subjects(&self) -> Result<Subjects> {
        Ok(self
            .fetch_synergia_endpoint(AuthenticatedSynergiaEndpoints::Subjects)
            .await?)
    }

    pub async fn categories(&self) -> Result<Categories> {
        Ok(self
            .fetch_synergia_endpoint(AuthenticatedSynergiaEndpoints::Categories)
            .await?)
    }

    pub async fn comment(&self, id: CommentId) -> Result<Comment> {
        Ok(self
            .fetch_synergia_endpoint::<RawComment>(AuthenticatedSynergiaEndpoints::Comments(id))
            .await?
            .comment)
    }
}
