use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::repositories::account_selection as models;

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
pub struct LibrusApiToken(String);

impl LibrusApiToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibrusApiTokenEntry {
    pub id: SynergiaUserId,
    pub access_token: LibrusApiToken,
}

#[derive(Serialize, Deserialize)]
struct RawLibrusApiTokens {
    accounts: Vec<LibrusApiTokenEntry>,
}

impl From<RawLibrusApiTokens> for LibrusApiTokens {
    fn from(value: RawLibrusApiTokens) -> Self {
        let tokens = value
            .accounts
            .into_iter()
            .map(|entry| (entry.id, entry.access_token))
            .collect::<HashMap<SynergiaUserId, LibrusApiToken>>();
        Self { inner: tokens }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "RawLibrusApiTokens")]
pub struct LibrusApiTokens {
    inner: HashMap<SynergiaUserId, LibrusApiToken>,
}

impl LibrusApiTokens {
    pub fn into_inner(self) -> HashMap<SynergiaUserId, LibrusApiToken> {
        self.inner
    }
}

#[derive(Clone, Debug)]
pub struct SynergiaToken(cookie::Cookie<'static>);

impl SynergiaToken {
    pub const NAME: &str = "oauth_token";

    pub fn new(_0: cookie::Cookie<'static>) -> Self {
        Self(_0)
    }

    pub fn as_inner(&self) -> &cookie::Cookie<'static> {
        &self.0
    }

    pub fn to_cookie_string(&self) -> String {
        self.0.encoded().stripped().to_string()
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
}

#[derive(Clone, Debug)]
pub struct PowerCookie(cookie::Cookie<'static>);

impl PowerCookie {
    pub const NAME: &'static str = "DZIENNIKSID";

    pub fn new(cookie: cookie::Cookie<'static>) -> Self {
        Self(cookie)
    }

    pub fn as_inner(&self) -> &cookie::Cookie<'static> {
        &self.0
    }

    pub fn into_inner(self) -> cookie::Cookie<'static> {
        self.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoLoginToken(String);

impl AutoLoginToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AutoLoginResponse {
    pub token: AutoLoginToken,
}
