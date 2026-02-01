use cookie::Cookie;
use serde::{Deserialize, Serialize};
use std::{cell::LazyCell, collections::HashMap};
use url::{form_urlencoded, Url};

use crate::{net::synergia_api::MESSAGES_URL, repositories::account_selection as models};

#[derive(Serialize, Deserialize)]
struct ClientId(usize);

impl ClientId {
    fn new() -> Self {
        Self(46)
    }
}

#[derive(Serialize)]
pub struct AuthorizationRequest {
    client_id: ClientId,
    response_type: String,
    scope: String,
}

impl AuthorizationRequest {
    // it's them who fucking fixed authorization and authentication into a single endpoint and
    // decided to call it authorization
    pub fn new() -> Self {
        Self {
            client_id: ClientId::new(),
            response_type: "code".to_owned(),
            scope: "mydata".to_owned(),
        }
    }
}

#[derive(Serialize)]
pub struct CaptchaRequest {
    username: String,
    is_needed: usize,
}

impl CaptchaRequest {
    pub fn new(username: String) -> Self {
        Self {
            username,
            is_needed: 1,
        }
    }
}

#[derive(Deserialize)]
pub struct CaptchaResponse {
    pub is_needed: bool,
    #[serde(rename = "reCaptchaCode")] // god I'm so glad that they consitently name things
    pub re_captcha_code: String,
}

// TODO: change this to correct after veryfying
#[derive(Serialize)]
pub struct NewLoginRequestParams {
    client_id: ClientId,
}

impl NewLoginRequestParams {
    pub fn new() -> Self {
        Self {
            client_id: ClientId::new(),
        }
    }
}

#[derive(Serialize)]
pub struct NewLoginRequestBody {
    action: String,
    login: String,
    pass: String,
}

impl NewLoginRequestBody {
    pub fn new(login: String, password: String) -> Self {
        Self {
            action: "login".to_owned(),
            login,
            pass: password,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLoginResponse {
    pub status: String,
    pub go_to: Option<String>,
}

#[derive(Debug)]
pub struct AuthToken(Cookie<'static>);

impl AuthToken {
    pub const NAME: &'static str = "oauth_token";

    pub fn new(cookie: Cookie<'static>) -> Self {
        Self(cookie)
    }
}

#[derive(Debug)]
pub struct PowerCookie(Cookie<'static>);

impl PowerCookie {
    pub const NAME: &'static str = "DZIENNIKSID";

    pub fn new(cookie: Cookie<'static>) -> Self {
        Self(cookie)
    }

    pub fn into_inner(self) -> Cookie<'static> {
        self.0
    }

    pub fn to_cookie_string(&self) -> String {
        self.0.encoded().stripped().to_string()
    }
}

#[derive(Debug)]
pub struct Authentication {
    pub auth_token: AuthToken,
    pub power_cookie: PowerCookie,
}

impl Authentication {
    pub fn new(auth_token: AuthToken, power_cookie: PowerCookie) -> Self {
        Self {
            auth_token,
            power_cookie,
        }
    }
}

#[derive(Debug)]
pub struct SecondFactorGoTo(String);

impl SecondFactorGoTo {
    pub fn new(_0: String) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug)]
pub enum LoginAttrKinds {
    RedirectTo,
    RedirectCrc,
    Token,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginAttrs {
    pub redirect_to: String,
    pub redirect_crc: String,
    #[serde(rename = "_token")]
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(flatten)]
    pub attrs: LoginAttrs,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SynergiaUserId(usize);

impl From<SynergiaUserId> for models::SynergiaUserId {
    fn from(value: SynergiaUserId) -> Self {
        Self::new(value.0)
    }
}

impl From<models::SynergiaUserId> for SynergiaUserId {
    fn from(value: models::SynergiaUserId) -> Self {
        Self(value.into_inner())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SynergiaAccount {
    pub id: SynergiaUserId,
    pub group: String,
    pub login: String,
    pub student_name: String,
    pub state: String,
}

impl From<SynergiaAccount> for models::SynergiaAccount {
    fn from(value: SynergiaAccount) -> Self {
        Self {
            id: value.id.into(),
            group: value.group,
            login: value.login,
            student_name: value.student_name,
            state: value.state,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SynergiaAccounts {
    pub accounts: Vec<SynergiaAccount>,
}

impl From<SynergiaAccounts> for models::SynergiaAccounts {
    fn from(value: SynergiaAccounts) -> Self {
        value.accounts.into_iter().map(Into::into).collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct PortalAccessToken(String);

impl PortalAccessToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct PortalRefreshToken(String);

impl PortalRefreshToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortalTokenPair {
    pub access_token: PortalAccessToken,
    pub refresh_token: PortalRefreshToken,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SynergiaToken(String);

impl SynergiaToken {
    const NAME: &str = "oauth_token";

    pub fn as_inner(&self) -> &str {
        &self.0
    }

    pub fn into_cookie_string(self) -> String {
        let value = form_urlencoded::byte_serialize(self.0.as_bytes()).collect::<String>();
        format!("{}={}", Self::NAME, value)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SynergiaTokenEntry {
    pub id: SynergiaUserId,
    pub access_token: SynergiaToken,
}

#[derive(Serialize, Deserialize)]
struct RawSynergiaTokens {
    accounts: Vec<SynergiaTokenEntry>,
}

impl From<RawSynergiaTokens> for SynergiaTokens {
    fn from(value: RawSynergiaTokens) -> Self {
        let tokens = value
            .accounts
            .into_iter()
            .map(|entry| (entry.id, entry.access_token))
            .collect::<HashMap<SynergiaUserId, SynergiaToken>>();
        Self { inner: tokens }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "RawSynergiaTokens")]
pub struct SynergiaTokens {
    inner: HashMap<SynergiaUserId, SynergiaToken>,
}

impl SynergiaTokens {
    pub fn inner(&self) -> &HashMap<SynergiaUserId, SynergiaToken> {
        &self.inner
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct AuthCode(String);

impl AuthCode {
    pub fn new(code: String) -> Self {
        Self(code)
    }
    pub fn as_inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Tokens {
    pub portal_token_pair: PortalTokenPair,
    pub synergia_tokens: SynergiaTokens,
}
