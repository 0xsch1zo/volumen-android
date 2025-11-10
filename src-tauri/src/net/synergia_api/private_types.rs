use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

// TODO: figure out whre to put types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortalTokenPair {
    pub access_token: PortalAccessToken,
    pub refresh_token: PortalRefreshToken,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SynergiaUserId(usize);

impl SynergiaUserId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SynergiaToken(String);

impl SynergiaToken {
    pub fn as_inner(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SynergiaAccount {
    pub id: SynergiaUserId,
    pub gruop: String,
    pub login: String,
    pub student_name: String,
    pub state: String,
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
