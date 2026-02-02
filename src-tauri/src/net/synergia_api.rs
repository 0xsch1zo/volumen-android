use std::{borrow::Cow, cell::LazyCell, sync::Arc, time::Duration};

use cookie_store::CookieStore;
use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    header::{InvalidHeaderValue, ToStrError},
    redirect::Policy,
    Url,
};
use reqwest_cookie_store::CookieStoreRwLock;
use scraper::{Html, Selector};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    error::{self, IntoStatefulErrorExt},
    net::{
        self,
        synergia_api::{
            account_selector::{AccountSelector, AccountSelectorError},
            api::{
                auth::{
                    AuthCode, AuthorizationRequest, CaptchaRequest, CaptchaResponse,
                    LoginAttrKinds, LoginAttrs, LoginRequest, NewLoginRequestBody,
                    NewLoginRequestParams, NewLoginResponse,
                },
                grades::{CategoriesResponse, CommentResponse, GradesResponse},
                //messages::Message,
                subjects::SubjectsResponse,
                users::UsersResponse,
            },
            auth_manager::AuthorizationManager,
            clients::{
                MainAuthenticatedClient,
                /*MessagesAuthenticatedClient,*/ UnauthenticatedClient,
            },
            token_management::{TokensApi, TokensApiError},
        },
    },
    repositories::{
        grades::{Categories, Comment, CommentId, ShallowGrades},
        subjects::Subjects,
        users::Users,
    },
    stateful_result,
};

pub mod account_selector;
mod api;
mod auth_manager;
mod clients;
pub mod token_management;

pub use api::messages::Message; // TODO: remove this after creating a repository for messages

const PORTAL_URL: LazyCell<Url> = LazyCell::new(|| Url::parse("https://portal.librus.pl").unwrap());

const SYNERGIA_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://synergia.librus.pl").unwrap());

const MESSAGES_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://wiadomosci.librus.pl").unwrap());

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
    #[error("token fetch error")]
    TokenFetchError(#[source] TokensApiError),
    #[error("account selector construction error")]
    AccountSelectorConstructionError(#[source] AccountSelectorError),
    #[error("unauthenticated client construction failure")]
    UnauthenticatedClientConstructionError(#[source] clients::ClientConstructionError),
    #[error("main authenticated client construction failure")]
    MainAuthenticatedClientConstructionError(#[source] clients::ClientConstructionError),
    #[error("main authenticated client construction failure")]
    MessagesAuthenticatedClientConstructionError(#[source] clients::ClientConstructionError),
    #[error("captcha request failure")]
    CaptchaError,
    #[error("falied to login")]
    LoginError,
    #[error("login returned status ok, but no goTo")]
    LoginWrongStateError,
}

type Result<T, E = Error> = std::result::Result<T, E>;

type StatefulError<S, E = Error> = error::StatefulError<S, E>;

pub trait ApiState {}

#[derive(Debug)]
pub struct UnauthenticatedState {
    client: UnauthenticatedClient,
}

impl UnauthenticatedState {
    fn try_new() -> Result<Self> {
        Ok(Self {
            client: UnauthenticatedClient::try_new(
                SynergiaApi::<UnauthenticatedState>::redirect_policy(),
            )
            .map_err(Error::UnauthenticatedClientConstructionError)?,
        })
    }
}

struct UnauthenticatedRedirectPolicy(Policy);

impl UnauthenticatedRedirectPolicy {
    fn into_inner(self) -> Policy {
        self.0
    }
}

#[derive(Debug)]
pub struct AuthenticatedState {
    main_client: MainAuthenticatedClient,
    //messages_client: MessagesAuthenticatedClient, // we use a different client because auth works
    //                                              // differently
}

impl AuthenticatedState {
    fn try_from_auth_manager(
        authorization_manager: AuthorizationManager,
    ) -> Result<Self, StatefulError<AuthorizationManager>> {
        let authorization_manager = Arc::new(authorization_manager);
        let main_client = MainAuthenticatedClient::try_new(Arc::clone(&authorization_manager))
            .map_err(Error::MainAuthenticatedClientConstructionError);
        let main_client = match main_client {
            Ok(c) => c,
            Err(e) => {
                // auth_manager should be dropped alredy
                return Err(e.into_stateful_err(Arc::into_inner(authorization_manager).unwrap()));
            }
        };

        /*let messages_client =
            MessagesAuthenticatedClient::try_new(cookie_store, Arc::clone(&authorization_manager))
                .map_err(Error::MessagesAuthenticatedClientConstructionError);
        let messages_client = match messages_client {
            Ok(c) => c,
            Err(e) => {
                drop(main_client);
                // auth_manager should be dropped alredy
                return Err(e.into_stateful_err(Arc::into_inner(authorization_manager).unwrap()));
            }
        };*/

        Ok(Self {
            main_client,
            //messages_client,
        })
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
            .as_inner()
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
            .as_inner()
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
                .map(AuthCode::new)
        };

        debug!("successfully logged in");

        let tokens_api = TokensApi::new();
        let tokens = stateful_result! { self =>
            tokens_api
                .fetch_tokens(auth_code)
                .await
                .map_err(Error::TokenFetchError)
        };

        Ok(stateful_result! { self =>
            AccountSelector::try_new(tokens, tokens_api)
                .map_err(Error::AccountSelectorConstructionError)
        })
    }

    async fn catpcha(&self, username: &str) -> Result<()> {
        debug!("initiating captcha");
        let req = CaptchaRequest::new(username.to_owned());

        let resp = self
            .state
            .client
            .as_inner()
            .post(UnauthenticatedSynergiaEndpoints::Captcha.url())
            .form(&req)
            .send()
            .await?
            .json::<CaptchaResponse>()
            .await?;

        match resp {
            CaptchaResponse {
                is_needed: false, ..
            } => {
                debug!("successfully completed captcha");
                Ok(())
            }
            CaptchaResponse {
                is_needed: true, ..
            } => Err(Error::CaptchaError),
        }
    }

    async fn authorize(&self) -> Result<()> {
        debug!("authorizing");

        let req = AuthorizationRequest::new();

        self.state
            .client
            .as_inner()
            .get(UnauthenticatedSynergiaEndpoints::Authorization.url())
            .query(&req)
            .send()
            .await?;
        Ok(())
    }

    pub async fn new_login(&self, login: &str, password: &str) -> Result<()> {
        debug!("logging in");

        self.authorize().await?;

        self.catpcha(login).await?;

        let req_body = NewLoginRequestBody::new(login.to_owned(), password.to_owned());

        let resp = self
            .state
            .client
            .as_inner()
            .post(UnauthenticatedSynergiaEndpoints::Authorization.url())
            .query(&NewLoginRequestParams::new())
            .form(&req_body)
            .send()
            .await?
            .json::<NewLoginResponse>()
            .await?;

        let go_to = match resp {
            NewLoginResponse {
                go_to: Some(go_to),
                status,
            } if status.as_str() == "ok" => go_to,
            NewLoginResponse {
                go_to: None,
                status,
            } if status.as_str() == "ok" => return Err(Error::LoginWrongStateError),
            _ => return Err(Error::LoginError),
        };

        debug!("successfully completed first login step");

        self.state
            .client
            .as_inner()
            .get(LIBRUS_API_URL.join(&go_to).unwrap())
            .send()
            .await?;
        Ok(())
    }
}

enum UnauthenticatedSynergiaEndpoints {
    Captcha,
    Authorization,
}

impl UnauthenticatedSynergiaEndpoints {
    fn url(&self) -> Url {
        let endpoint = match self {
            UnauthenticatedSynergiaEndpoints::Captcha => "/OAuth/Captcha",
            UnauthenticatedSynergiaEndpoints::Authorization => "/OAuth/Authorization",
        };
        LIBRUS_API_URL.join(endpoint).unwrap()
    }
}

#[derive(Debug)]
enum AuthenticatedSynergiaEndpoints {
    Me,
    Users,
    Subjects,
    Grades(GradesEndpoints),
    Messages(MessagesEndpoints),
}

impl AuthenticatedSynergiaEndpoints {
    fn url(&self) -> Url {
        match self {
            AuthenticatedSynergiaEndpoints::Me => SYNERGIA_URL.join("/gateway/api/2.0/Me").unwrap(),
            AuthenticatedSynergiaEndpoints::Users => {
                SYNERGIA_URL.join("/gateway/api/2.0/Users").unwrap()
            }
            AuthenticatedSynergiaEndpoints::Subjects => {
                SYNERGIA_URL.join("/gateway/api/2.0/Subjects").unwrap()
            }
            AuthenticatedSynergiaEndpoints::Grades(grades) => grades.url(),
            AuthenticatedSynergiaEndpoints::Messages(messages) => messages.url(),
        }
    }
}

#[derive(Debug)]
enum GradesEndpoints {
    Grades,
    Categories,
    Comments(CommentId),
}

impl GradesEndpoints {
    fn url(&self) -> Url {
        let endoint = match self {
            GradesEndpoints::Grades => "/gateway/api/2.0/Grades",
            GradesEndpoints::Categories => "/gateway/api/2.0/Grades/Categories",
            GradesEndpoints::Comments(id) => {
                &format!("/gateway/api/2.0/Grades/Comments/{}", id.inner())
            }
        };
        SYNERGIA_URL.join(endoint).unwrap()
    }
}

#[derive(Debug)]
enum MessagesEndpoints {
    Authorization,
    Recieved { page: usize, limit: usize },
    Sent { page: usize, limit: usize },
}

impl MessagesEndpoints {
    fn url(&self) -> Url {
        let endpoint = match self {
            MessagesEndpoints::Recieved { page, limit } => {
                &format!("/api/inbox/messages?page={page}&limit={limit}")
            }
            MessagesEndpoints::Sent { page, limit } => {
                &format!("/api/outbox/messages?page={page}&limit={limit}")
            }
            MessagesEndpoints::Authorization => return SYNERGIA_URL.join("/wiadomosci3").unwrap(),
        };
        MESSAGES_URL.join(endpoint).unwrap()
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
            .main_client
            .as_inner()
            .get(endpoint.url())
            .send()
            .await?
            .json::<T>()
            .await?;
        debug!("fetched {endpoint:?} succesfully");
        Ok(resource)
    }

    pub async fn fetch_users(&self) -> Result<Users> {
        Ok(self
            .fetch_synergia_endpoint::<UsersResponse>(AuthenticatedSynergiaEndpoints::Users)
            .await?
            .into())
    }

    pub async fn fetch_subjects(&self) -> Result<Subjects> {
        Ok(self
            .fetch_synergia_endpoint::<SubjectsResponse>(AuthenticatedSynergiaEndpoints::Subjects)
            .await?
            .into())
    }

    pub fn grades(&self) -> GradesManager {
        GradesManager::new(&self)
    }

    pub fn messages(&self) -> MessagesManager {
        MessagesManager::new(&self)
    }
}

pub struct GradesManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> GradesManager<'a> {
    fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    pub async fn fetch_self(&self) -> Result<ShallowGrades> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<GradesResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Grades,
            ))
            .await?
            .into())
    }

    pub async fn fetch_categories(&self) -> Result<Categories> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<CategoriesResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Categories,
            ))
            .await?
            .into())
    }

    pub async fn fetch_comment(&self, id: CommentId) -> Result<Comment> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<CommentResponse>(AuthenticatedSynergiaEndpoints::Grades(
                GradesEndpoints::Comments(id),
            ))
            .await?
            .into())
    }
}

pub struct MessagesManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> MessagesManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    /*pub async fn feth_message_endpoint<T: DeserializeOwned>(&self endpoint: AuthenticatedSynergiaEndpoints) -> Result<T> {
    w        debug!("fetching {endpoint:?}");
            let resource = self
                .synergia_api
                .state
                .m
                .as_inner()
                .get(endpoint.url())
                .send()
                .await?
                .json::<T>()
                .await?;
            debug!("fetched {endpoint:?} succesfully");
            Ok(resource)
        }
        }*/

    pub async fn fetch_recieved(&self) -> Result<Vec<Message>> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<Vec<Message>>(AuthenticatedSynergiaEndpoints::Messages(
                MessagesEndpoints::Recieved { page: 1, limit: 10 },
            ))
            .await?)
    }

    pub async fn fetch_sent(&self) -> Result<Vec<Message>> {
        Ok(self
            .synergia_api
            .fetch_synergia_endpoint::<Vec<Message>>(AuthenticatedSynergiaEndpoints::Messages(
                MessagesEndpoints::Sent { page: 1, limit: 10 },
            ))
            .await?)
    }
}
