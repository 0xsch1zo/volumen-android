use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::net::synergia_api::api::Reference;
use crate::repositories::users as models;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(from = "Reference")]
pub(super) struct UserId(usize);

impl From<Reference> for UserId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<UserId> for models::UserId {
    fn from(value: UserId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, From)]
#[serde(rename_all = "PascalCase")]
pub(super) struct User {
    id: UserId,
    first_name: String,
    last_name: String,
}

impl From<User> for models::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id.into(),
            first_name: value.first_name,
            last_name: value.last_name,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UsersResponse {
    users: Vec<User>,
}

impl From<UsersResponse> for models::Users {
    fn from(value: UsersResponse) -> Self {
        value.users.into_iter().map(Into::into).collect()
    }
}
