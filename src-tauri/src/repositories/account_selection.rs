use crate::{
    error::StatefulResultExt,
    net::synergia_api::account_selector::AccountSelector,
    repositories::{
        entities::{SynergiaAccounts, SynergiaUserId},
        main::MainRepository,
        Result, StatefulError,
    },
};

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
