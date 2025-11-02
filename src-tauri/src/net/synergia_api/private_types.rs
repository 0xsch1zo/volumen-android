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
pub struct UserId(usize);

impl UserId {
    fn inner(&self) -> usize {
        self.0
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
pub struct SynergiaAccount {
    pub id: UserId,
    pub gruop: String,
    pub login: String,
    pub student_name: String,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
struct SynergiaTokenEntry {
    pub id: UserId,
    pub access_token: SynergiaToken,
}

#[derive(Serialize, Deserialize)]
struct RawSynergiaTokens {
    tokens: Vec<SynergiaTokenEntry>,
}

impl From<RawSynergiaTokens> for SynergiaTokens {
    fn from(value: RawSynergiaTokens) -> Self {
        let tokens = value
            .tokens
            .into_iter()
            .map(|entry| (entry.id, entry.access_token))
            .collect::<HashMap<UserId, SynergiaToken>>();
        Self { inner: tokens }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "RawSynergiaTokens")]
pub struct SynergiaTokens {
    inner: HashMap<UserId, SynergiaToken>,
}

impl SynergiaTokens {
    pub fn inner(&self) -> &HashMap<UserId, SynergiaToken> {
        &self.inner
    }
}
