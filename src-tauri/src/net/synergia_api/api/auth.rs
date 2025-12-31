use serde::{Deserialize, Serialize};
use std::{cell::LazyCell, collections::HashMap};
use url::{form_urlencoded, Url};

use crate::{net::synergia_api::MESSAGES_URL, repositories::account_selection as models};
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

#[derive(Debug)]
pub struct PowerCookie(cookie_store::Cookie<'static>);

impl PowerCookie {
    pub const NAME: &'static str = "DZIENNIKSID";
    pub const DOMAIN: &'static str = "wiadomosci.librus.pl";
    pub const PATH: &'static str = "/";
    pub const URL: LazyCell<Url> = MESSAGES_URL;

    pub fn new(cookie: cookie_store::Cookie<'static>) -> Self {
        Self(cookie)
    }

    pub fn into_inner(self) -> cookie_store::Cookie<'static> {
        self.0
    }

    pub fn to_cookie_string(&self) -> String {
        self.0.encoded().stripped().to_string()
    }
}
