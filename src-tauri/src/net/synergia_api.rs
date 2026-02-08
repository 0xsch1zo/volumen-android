use std::{borrow::Cow, cell::LazyCell, sync::Arc};

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    header::{InvalidHeaderValue, ToStrError},
    redirect::Policy,
    Url,
};
use scraper::{Html, Selector};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    error,
    net::{
        self,
        synergia_api::{
            account_selector::{AccountSelector, AccountSelectorConstructionError},
            api::{
                auth::{
                    AuthCode, LoginAttrKinds, LoginAttrs, LoginRequest, PortalTokenPair,
                    SynergiaUserId,
                },
                grades::{CategoriesResponse, CommentResponse, GradesResponse},
                messages::{received, sent, MessageModelConversionError},
                subjects::SubjectsResponse,
                users::UsersResponse,
            },
            authenticators::{MainAuthenticator, MainAuthenticatorError},
            clients::{MainAuthenticatedClient, MessagesClient, UnauthenticatedClient},
            credential_manager::{PortalCredentialFetchError, PortalCredentialManager},
        },
    },
    repositories::{
        grades::{Categories, Comment, CommentId, ShallowGrades},
        messages::{
            received::{ReceivedMessage, ReceivedMessagePreviews},
            sent::{SentMessage, SentMessagePreviews},
            Limit, MessageId, Page,
        },
        subjects::Subjects,
        users::Users,
    },
    stateful_result,
};

pub mod account_selector;
mod api;
mod authenticators;
mod clients;
pub mod credential_manager;

const PORTAL_URL: LazyCell<Url> = LazyCell::new(|| Url::parse("https://portal.librus.pl").unwrap());

const SYNERGIA_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://synergia.librus.pl").unwrap());

const LIBRUS_API_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://api.librus.pl").unwrap());

const MESSAGES_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://wiadomosci.librus.pl").unwrap());

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
    #[error("account selector construction error")]
    AccountSelectorConstructionError(#[source] AccountSelectorConstructionError),
    #[error("main authenticator init error")]
    MainAuthenticatorInitError(#[source] MainAuthenticatorError),
    #[error("unauthenticated client construction failure")]
    UnauthenticatedClientConstructionError(
        #[source] clients::UnauthenticatedClientConstructionError,
    ),
    #[error("main authenticated client construction failure")]
    MainAuthenticatedClientConstructionError(
        #[source] clients::AuthenticatedClientConstructionError,
    ),
    #[error("main authenticated client construction failure")]
    MessagesAuthenticatedClientConstructionError(#[source] clients::MessagesClientInitError),
    #[error("portal cred manager construciton error")]
    PortalCredManagerConstructionError(#[source] credential_manager::PortalClientConstructionError),
    #[error("portal credential fetch errro")]
    PortalCredFetchError(#[source] PortalCredentialFetchError),
    #[error("model conversion error")]
    ModelConversionError(#[from] ModelConversionError),
}

#[derive(Error, Debug)]
pub enum ModelConversionError {
    #[error("message model conversion error")]
    MessageConvError(#[from] MessageModelConversionError),
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
    messages_client: MessagesClient, // we use a different client because auth works
                                     // differently
}

impl AuthenticatedState {
    async fn init(
        user_id: SynergiaUserId,
        portal_creds: PortalTokenPair,
    ) -> Result<Self, StatefulError<(SynergiaUserId, PortalTokenPair)>> {
        let main_authenticator = stateful_result! { (user_id, portal_creds) =>
            MainAuthenticator::init(user_id, portal_creds.clone())
                    .await
                    .map(Arc::new)
                    .map_err(Error::MainAuthenticatorInitError)
        };

        let main_client = stateful_result! { (user_id, portal_creds) =>
            MainAuthenticatedClient::try_new(Arc::clone(&main_authenticator))
                .map_err(Error::MainAuthenticatedClientConstructionError)
        };

        let messages_client = stateful_result! { (user_id, portal_creds) =>
            MessagesClient::init(Arc::clone(&main_authenticator))
                .await
                .map_err(Error::MessagesAuthenticatedClientConstructionError)
        };

        Ok(Self {
            main_client,
            messages_client,
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
        email: String,
        password: String,
    ) -> Result<AccountSelector, StatefulError<Self>> {
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
    CurrentMessages(MessagesSourceEndpoints),
    ArchivedMessages(MessagesSourceEndpoints),
}

impl MessagesEndpoints {
    fn url(&self) -> Url {
        match self {
            MessagesEndpoints::Authorization => SYNERGIA_URL.join("/wiadomosci3").unwrap(),
            MessagesEndpoints::CurrentMessages(endpoints) => endpoints.current_url(),
            MessagesEndpoints::ArchivedMessages(endpoints) => endpoints.archive_url(),
        }
    }
}

#[derive(Debug)]
enum MessagesSourceEndpoints {
    ReceivedMessage { id: usize },
    ReceivedMessages { page: usize, limit: usize },
    SentMessage { id: usize },
    SentMessages { page: usize, limit: usize },
}

impl MessagesSourceEndpoints {
    fn current_url(&self) -> Url {
        let endpoint = match self {
            MessagesSourceEndpoints::ReceivedMessage { id } => &format!("/api/inbox/messages/{id}"),
            MessagesSourceEndpoints::ReceivedMessages { page, limit } => {
                &format!("/api/inbox/messages?page={page}&limit={limit}")
            }
            MessagesSourceEndpoints::SentMessage { id } => &format!("/api/outbox/messages/{id}"),
            MessagesSourceEndpoints::SentMessages { page, limit } => {
                &format!("/api/outbox/messages?page={page}&limit={limit}")
            }
        };
        MESSAGES_URL.join(endpoint).unwrap()
    }

    fn archive_url(&self) -> Url {
        let endpoint = match self {
            MessagesSourceEndpoints::ReceivedMessage { id } => {
                &format!("/api/archive/inbox/messages/{id}")
            }
            MessagesSourceEndpoints::ReceivedMessages { page, limit } => {
                &format!("/api/archive/inbox/messages?page={page}&limit={limit}")
            }
            MessagesSourceEndpoints::SentMessage { id } => {
                &format!("/api/archive/outbox/messages/{id}")
            }
            MessagesSourceEndpoints::SentMessages { page, limit } => {
                &format!("/api/archive/outbox/messages?page={page}&limit={limit}")
            }
        };
        MESSAGES_URL.join(endpoint).unwrap()
    }
}

// We're using the api of the new ui
impl SynergiaApi<AuthenticatedState> {
    async fn init(
        user_id: SynergiaUserId,
        portal_creds: PortalTokenPair,
    ) -> Result<Self, StatefulError<(SynergiaUserId, PortalTokenPair)>> {
        Ok(Self {
            state: AuthenticatedState::init(user_id, portal_creds).await?,
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

enum MessagesSource {
    Current,
    Archive,
}

impl MessagesSource {
    fn into_messages_endpoint(
        &self,
        source_endpoint: MessagesSourceEndpoints,
    ) -> MessagesEndpoints {
        match self {
            Self::Current => MessagesEndpoints::CurrentMessages(source_endpoint),
            Self::Archive => MessagesEndpoints::ArchivedMessages(source_endpoint),
        }
    }
}

pub struct MessagesManager<'a> {
    synergia_api: &'a SynergiaApi<AuthenticatedState>,
}

impl<'a> MessagesManager<'a> {
    pub fn new(synergia_api: &'a SynergiaApi<AuthenticatedState>) -> Self {
        Self { synergia_api }
    }

    async fn fetch_message_endpoint<T: DeserializeOwned>(
        &self,
        endpoint: AuthenticatedSynergiaEndpoints,
    ) -> Result<T> {
        debug!("fetching message endpoint: {endpoint:?}");
        let resource = self
            .synergia_api
            .state
            .messages_client
            .as_inner()
            .get(endpoint.url())
            .send()
            .await?
            .json::<T>()
            .await?;
        debug!("fetched message endpoint: {endpoint:?} succesfully");
        Ok(resource)
    }

    pub fn received(&self) -> ReceivedMessagesManager {
        ReceivedMessagesManager::new(self, MessagesSource::Current)
    }

    pub fn sent(&self) -> SentMessagesManager {
        SentMessagesManager::new(self, MessagesSource::Current)
    }

    pub fn archive(&self) -> MessagesArchiveManager {
        MessagesArchiveManager::new(self)
    }
}

pub struct ReceivedMessagesManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
    source: MessagesSource,
}

impl<'a> ReceivedMessagesManager<'a> {
    fn new(messages_manager: &'a MessagesManager<'a>, source: MessagesSource) -> Self {
        Self {
            messages_manager,
            source,
        }
    }

    pub async fn fetch_list(&self, page: Page, limit: Limit) -> Result<ReceivedMessagePreviews> {
        let source_endopint = MessagesSourceEndpoints::ReceivedMessages {
            page: page.into_inner(),
            limit: limit.into_inner(),
        };

        Ok(self
            .messages_manager
            .fetch_message_endpoint::<received::ReceivedMessagePreviews>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endopint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }

    pub async fn fetch_message(&self, message_id: MessageId) -> Result<ReceivedMessage> {
        let source_endpoint = MessagesSourceEndpoints::ReceivedMessage {
            id: message_id.into_inner(),
        };

        Ok(self
            .messages_manager
            .fetch_message_endpoint::<received::ReceivedMessageResponse>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }
}

pub struct SentMessagesManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
    source: MessagesSource,
}

impl<'a> SentMessagesManager<'a> {
    fn new(messages_manager: &'a MessagesManager, source: MessagesSource) -> Self {
        Self {
            messages_manager,
            source,
        }
    }

    pub async fn fetch_list(&self, page: Page, limit: Limit) -> Result<SentMessagePreviews> {
        let source_endpoint = MessagesSourceEndpoints::SentMessages {
            page: page.into_inner(),
            limit: limit.into_inner(),
        };
        Ok(self
            .messages_manager
            .fetch_message_endpoint::<sent::SentMessagePreviews>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }

    pub async fn fetch_message(&self, message_id: MessageId) -> Result<SentMessage> {
        let source_endpoint = MessagesSourceEndpoints::SentMessage {
            id: message_id.into_inner(),
        };
        Ok(self
            .messages_manager
            .fetch_message_endpoint::<sent::SentMessageResponse>(
                AuthenticatedSynergiaEndpoints::Messages(
                    self.source.into_messages_endpoint(source_endpoint),
                ),
            )
            .await?
            .try_into()
            .map_err(ModelConversionError::from)?)
    }
}

pub struct MessagesArchiveManager<'a> {
    messages_manager: &'a MessagesManager<'a>,
}

impl<'a> MessagesArchiveManager<'a> {
    pub fn new(messages_manager: &'a MessagesManager<'a>) -> Self {
        Self { messages_manager }
    }

    pub fn received(&self) -> ReceivedMessagesManager {
        ReceivedMessagesManager::new(&self.messages_manager, MessagesSource::Archive)
    }

    pub fn sent(&self) -> SentMessagesManager {
        SentMessagesManager::new(&self.messages_manager, MessagesSource::Archive)
    }
}
