use serde::Serialize;

use crate::{
    error::StatefulResultExt,
    net::synergia_api::account_selector::AccountSelector,
    repositories::{main::MainRepository, Result, StatefulError},
};

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SynergiaUserId(usize);

impl SynergiaUserId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SynergiaAccount {
    pub id: SynergiaUserId,
    pub group: String,
    pub login: String,
    pub student_name: String,
    pub state: String,
}

pub type SynergiaAccounts = Vec<SynergiaAccount>;

#[derive(Debug)]
pub struct AccountSelectionRepository {
    account_selector: AccountSelector,
}

impl AccountSelectionRepository {
    pub fn new(account_selector: AccountSelector) -> Self {
        Self { account_selector }
    }

    pub async fn accounts(&self) -> Result<SynergiaAccounts> {
        Ok(self.account_selector.accounts().await?)
    }

    pub fn select(self, user_id: SynergiaUserId) -> Result<MainRepository, StatefulError<Self>> {
        self.account_selector
            .select(user_id)
            .map(MainRepository::new)
            .map_err_state(AccountSelectionRepository::new)
            .map_stateful_err(Into::into)
    }
}
