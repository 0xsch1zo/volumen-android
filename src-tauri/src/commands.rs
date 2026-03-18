use tauri::State;

use crate::{
    error::{ApplicationResultExt, LoggedApplicationResultExt, StatefulResultExt},
    repositories::account_selection::{SynergiaAccount, SynergiaUserId},
    state::{AccountSelectionState, AppStates, AuthenticatedState, UnauthenticatedState},
    Result,
};

#[tauri::command]
async fn login(state: State<'_, AppStates>, login: String, password: String) -> Result<()> {
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<UnauthenticatedState, AccountSelectionState>(async |s| {
            let account_selection_repo = s
                .login_repo
                .login(login, password)
                .await
                .map_err_state(UnauthenticatedState::new)
                .map_stateful_err(Into::into)?;
            Ok(AccountSelectionState::new(account_selection_repo))
        })
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(())
}

#[tauri::command]
async fn accounts(state: State<'_, AppStates>) -> Result<Vec<SynergiaAccount>> {
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AccountSelectionState>()
        .into_app_result()
        .log_on_err()?;

    let accounts = state
        .account_selection_repo
        .accounts()
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(accounts)
}

#[tauri::command]
async fn select_account(state: State<'_, AppStates>, user_id: SynergiaUserId) -> Result<()> {
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<AccountSelectionState, AuthenticatedState>(async |s| {
            Ok(AuthenticatedState::new(
                s.account_selection_repo
                    .select(user_id)
                    .await
                    .map_err_state(AccountSelectionState::new)
                    .map_stateful_err(Into::into)?,
            ))
        })
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(())
}
