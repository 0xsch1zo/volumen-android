use std::env;

use log::LevelFilter;
use tauri::{async_runtime::Mutex, Manager, State};

use crate::{
    error::{ApplicationResultExt, FrontendError, LoggedApplicationResultExt},
    state::{
        AppStates, AppStatesInner, AuthenticatedState, StateTranstionError, UnauthenticatedState,
        UnselectedAccountState,
    },
};

mod common;
mod error;
mod net;
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
        .state_transition::<UnauthenticatedState, UnselectedAccountState>(async |s| {
            Ok(UnselectedAccountState {
                account_manager: s
                    .synergia_api
                    .clone()
                    .login(&login, &password)
                    .await
                    .into_app_result()
                    .map_err(|e| StateTranstionError::new(s, e))?,
            })
        })
        .await
        .into_app_result()
        .log_on_err()?;

    let state = state_lock
        .as_state::<AuthenticatedState>()
        .into_app_result()
        .log_on_err()?;
    let text = state
        .synergia_api
        .me()
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(text)
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
