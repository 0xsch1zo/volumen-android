use std::env;

use log::LevelFilter;
use tauri::{async_runtime::Mutex, Manager, State};

use crate::{
    error::{ApplicationResultExt, FrontendError, LoggedApplicationResultExt, StatefulResultExt},
    state::{
        AccountSelectionState, AppStates, AppStatesInner, AuthenticatedState, UnauthenticatedState,
    },
};

mod cache;
mod common;
mod error;
mod net;
mod repositories;
mod state;

#[cfg(debug_assertions)]
const fn is_debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
const fn is_debug() -> bool {
    false
}

type Result<T, E = FrontendError> = std::result::Result<T, E>;

#[tauri::command]
async fn send(state: State<'_, AppStates>, login: String, password: String) -> Result<String> {
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<UnauthenticatedState, AccountSelectionState>(async |s| {
            let account_selection_repo = s
                .login_repo
                .login(&login, &password)
                .await
                .map_err_state(UnauthenticatedState::new)
                .map_stateful_err(Into::into)?;
            Ok(AccountSelectionState::new(account_selection_repo))
        })
        .await
        .into_app_result()
        .log_on_err()?;

    let state = state_lock
        .as_state::<AccountSelectionState>()
        .into_app_result()
        .log_on_err()?;

    let accounts = state
        .account_selection_repo
        .accounts()
        .await
        .into_app_result()
        .log_on_err()?
        .inner;

    state_lock
        .state_transition::<AccountSelectionState, AuthenticatedState>(async |s| {
            Ok(AuthenticatedState::new(
                s.account_selection_repo
                    .select(accounts[0].id)
                    .map_err_state(AccountSelectionState::new)
                    .map_stateful_err(Into::into)?,
            ))
        })
        .await
        .into_app_result()
        .log_on_err()?;

    let state = state_lock
        .as_state::<AuthenticatedState>()
        .into_app_result()
        .log_on_err()?;
    let me = state.main_repo.me().await.into_app_result().log_on_err()?;
    Ok(format!("{:?}", me))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut logger = env_logger::builder();
    match env::var("RUST_LOG") {
        Err(env::VarError::NotPresent) => logger.filter_level(LevelFilter::Debug),
        _ => &mut logger,
    }
    .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = Mutex::new(AppStatesInner::try_new()?);
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![send])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
