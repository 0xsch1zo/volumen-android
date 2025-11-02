use std::{borrow::Cow, cell::LazyCell};

use itertools::Itertools;
use log::{debug, warn};
use reqwest::{
    header::{self, InvalidHeaderValue, ToStrError},
    redirect::{Action, Attempt, Policy},
    Request, Response, StatusCode, Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use scraper::{Html, Selector};
use tauri::http::{Extensions, HeaderValue};
use thiserror::Error;
use tokio::sync::RwLockReadGuard;

use crate::{
    common::TakeExactlyExt,
    net::{
        self,
        synergia_api::{
            private_types::{LoginAttrKinds, LoginAttrs, LoginRequest, UserId},
            token_manager::{AuthCode, TokenManager, Tokens},
        },
        ErrorStatusMiddleware, IsSameBaseExt,
    },
};

mod private_types;
mod public_types;
mod token_manager;

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
    #[error("token manager error")]
    TokenManagerError(#[from] token_manager::Error),
    #[error("synergia account not found")]
    SynergiaAccessTokenNotFound,
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct TokenPicker {
    synergia_id: UserId,
}

impl TokenPicker {
    fn new(synergia_id: UserId) -> Self {
        Self { synergia_id }
    }

    fn pick(&self, url: &Url, tokens: &Tokens) -> Result<Option<String>> {
        let managed_hosts = [
            (PORTAL_URL, tokens.portal_token_pair.access_token.as_inner()),
            (
                SYNERGIA_URL,
                tokens
                    .synergia_tokens
                    .inner()
                    .get(&self.synergia_id)
                    .ok_or(Error::SynergiaAccessTokenNotFound)?
                    .as_inner(),
            ),
        ];

        let Some(token) = managed_hosts
            .into_iter()
            .find(|other| url.is_same_base(&other.0))
        else {
            return Ok(None);
        };
        Ok(Some(token.1.to_owned()))
    }
}

struct AuthorizationMiddleware {
    token_manager: TokenManager,
    token_picker: TokenPicker,
}

impl AuthorizationMiddleware {
    fn new(token_manager: TokenManager, token_picker: TokenPicker) -> Self {
        Self {
            token_manager,
            token_picker,
        }
    }

    async fn add_auth_token_on_managed(&self, req: &mut Request) -> Result<()> {
        let tokens = self.token_manager.get().await;
        if let Some(token) = self.token_picker.pick(req.url(), &tokens)? {
            req.headers_mut()
                .insert(header::AUTHORIZATION, HeaderValue::from_str(&token)?);
        }
        Ok(())
    }

    async fn handle_unauthorized(
        &self,
        req: Request,
        res: Response,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let tokens_lock = self.token_manager.get().await;
        let tokens = tokens_lock.clone();
        drop(tokens_lock); // read lock ends here
        let is_managed = self.token_picker.pick(req.url(), &tokens)?.is_some();
        if is_managed && res.status() == StatusCode::UNAUTHORIZED {
            self.token_manager
                .refresh() // write lock acquire
                .await?;
            Ok(next.run(req, extensions).await?)
        } else {
            Ok(res)
        }
    }
}

#[async_trait::async_trait]
impl Middleware for AuthorizationMiddleware {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response, reqwest_middleware::Error> {
        self.add_auth_token_on_managed(&mut req)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;

        // https://github.com/TrueLayer/reqwest-middleware/blob/43d31fea66ba23774738d4518da2b4ad40fc346f/reqwest-retry/src/middleware.rs#L146-L149
        // TLDR: this clone should be cheap
        let Some(req_clone) = req.try_clone() else {
            return next.run(req, extensions).await;
        };

        let res = next.clone().run(req_clone, extensions).await?;
        let res = self
            .handle_unauthorized(req, res, extensions, next)
            .await
            .map_err(|e| reqwest_middleware::Error::middleware(e))?;
        Ok(res)
    }
}

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
    fn try_from_token_manager(token_manager: TokenManager) -> Result<Self> {
        Ok(Self {
            client: Self::build_client(token_manager)?,
        })
    }

    fn build_client(token_manager: TokenManager) -> Result<ClientWithMiddleware> {
        let reqwest_client = net::default_client_options().cookie_store(true).build()?;

        // TODO: add auth middleware
        Ok(ClientBuilder::new(reqwest_client)
            .with(ErrorStatusMiddleware)
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
        Ok(Self {
            state: AuthenticatedState::try_from_token_manager(token_manager)?,
        })
    }
}
