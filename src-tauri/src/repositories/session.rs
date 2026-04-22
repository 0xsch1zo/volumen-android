use std::sync::Arc;

use tauri::{AppHandle, Manager};
use thiserror::Error;

use crate::{
    cache::{CacheComputeError, SingleEntryCache},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::account_selection::SynergiaAccount,
    state::{self, AppStates},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to reset state at logout. we're cooked")]
    StateResetError(#[source] state::Error),
    #[error("failed to fetch user details")]
    MeError(#[source] CacheComputeError),
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: usize,
    pub user_id: usize,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone)]
pub struct ClassId(usize);

impl ClassId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }
}

#[derive(Debug, Clone)]
pub struct Me {
    pub account: Account,
    pub class: ClassId,
}

#[derive(Debug, Clone)]
pub struct SessionRepository {
    app_handle: AppHandle,
    current_account: SynergiaAccount,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    me_cache: SingleEntryCache<Me>,
}

impl SessionRepository {
    pub fn new(
        synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
        current_account: SynergiaAccount,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            current_account,
            app_handle: app_handle.clone(),
            synergia_api,
            me_cache: SingleEntryCache::new(),
        }
    }

    pub async fn me(&self) -> Result<Me, Error> {
        self.me_cache
            .try_get_with(async { self.synergia_api.fetch_me().await })
            .await
            .map_err(Error::MeError)
    }

    pub fn current_account(&self) -> SynergiaAccount {
        self.current_account.clone()
    }

    // maybe delegate resetting state to something else
    pub async fn logout(&self) -> Result<(), Error> {
        let state = self.app_handle.state::<AppStates>();
        let mut state_lock = state.lock().await;
        state_lock.reset().map_err(Error::StateResetError)?;
        todo!()
    }
}
