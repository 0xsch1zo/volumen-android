use std::sync::Arc;

use tauri::{AppHandle, Manager};
use thiserror::Error;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::account_selection::SynergiaAccount,
    state::{self, AppStates},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to reset state at logout. we're cooked")]
    StateResetError(#[source] state::Error),
}

#[derive(Debug, Clone)]
pub struct SessionRepository {
    app_handle: AppHandle,
    current_account: SynergiaAccount,
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
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
        }
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
