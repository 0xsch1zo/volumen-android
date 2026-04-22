use serde::Deserialize;

use crate::{net::synergia_api::api::Reference, repositories::session as models};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Account {
    id: usize,
    user_id: usize,
    first_name: String,
    last_name: String,
}

impl From<Account> for models::Account {
    fn from(value: Account) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}

#[derive(Deserialize)]
#[serde(from = "Reference")]
struct ClassId(usize);

impl From<Reference> for ClassId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<ClassId> for models::ClassId {
    fn from(value: ClassId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Me {
    account: Account,
    class: ClassId,
}

impl From<Me> for models::Me {
    fn from(value: Me) -> Self {
        Self {
            account: value.account.into(),
            class: value.class.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MeResponse {
    pub me: Me,
}
