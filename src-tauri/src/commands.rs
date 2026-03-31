use tauri::{AppHandle, Manager, State};

use crate::{
    error::{
        ApplicationError, ApplicationResultExt, LoggedApplicationResultExt, StatefulResultExt,
    },
    repositories::account_selection::{SynergiaAccount, SynergiaUserId},
    state::{AccountSelectionState, AppStates, StateTransitionError, UnauthenticatedState},
    sync::LogoutSignaler,
    Result,
};

#[tauri::command]
pub async fn login(state: State<'_, AppStates>, login: String, password: String) -> Result<()> {
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<UnauthenticatedState, AccountSelectionState>(async |s| {
            let account_selection_repo = s
                .login_repo
                .login(login, password)
                .await
                .map_err_state(UnauthenticatedState::from_repo)
                .map_stateful_err(StateTransitionError::LoginError)?;
            Ok(AccountSelectionState::new(account_selection_repo))
        })
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(())
}

#[tauri::command]
pub async fn accounts(app_handle: AppHandle) -> Result<Vec<SynergiaAccount>> {
    let logout_signaler = LogoutSignaler::new(app_handle.clone());
    let state = app_handle.state::<AppStates>();
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AccountSelectionState>()
        .map_err(ApplicationError::StateAquisitionError)
        .log_on_err()?;

    let accounts = state
        .account_selection_repo
        .accounts(logout_signaler)
        .await
        .map_err(ApplicationError::AccountListQueryError)
        .log_on_err()?;
    Ok(accounts)
}

#[tauri::command]
pub async fn select_account(app_handle: AppHandle, user_id: SynergiaUserId) -> Result<()> {
    let logout_signaler = LogoutSignaler::new(app_handle.clone());
    logout_signaler.send_logout_event();
    /*let state = app_handle.state::<AppStates>();
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<AccountSelectionState, AuthenticatedState>(async |s| {
            Ok(AuthenticatedState::new(
                s.account_selection_repo
                    .select(user_id, logout_signaler)
                    .await
                    .map_err_state(AccountSelectionState::new)
                    .map_stateful_err(Into::into)?,
            ))
        })
        .await
        .into_app_result()
        .log_on_err()?;*/
    Ok(())
}
