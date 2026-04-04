use futures::TryFutureExt;
use tauri::State;

use crate::{
    error::{
        ApplicationError, ApplicationResultExt, LoggedApplicationResultExt, StatefulResultExt,
    },
    repositories::{
        account_selection::{SynergiaAccount, SynergiaUserId},
        grades::Grade,
    },
    state::{
        AccountSelectionState, AppStates, AuthenticatedState, StateTransitionError,
        UnauthenticatedState,
    },
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
pub async fn accounts(state: State<'_, AppStates>) -> Result<Vec<SynergiaAccount>> {
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AccountSelectionState>()
        .map_err(ApplicationError::StateAquisitionError)
        .log_on_err()?;

    let accounts = state
        .account_selection_repo
        .accounts()
        .await
        .map_err(ApplicationError::AccountListQueryError)
        .log_on_err()?;
    Ok(accounts)
}

#[tauri::command]
pub async fn select_account(state: State<'_, AppStates>, user_id: SynergiaUserId) -> Result<()> {
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<AccountSelectionState, AuthenticatedState>(async |s| {
            Ok(AuthenticatedState::new(
                s.account_selection_repo
                    .select(user_id)
                    .await
                    .map_err_state(AccountSelectionState::new)
                    .map_stateful_err(StateTransitionError::AcccountSelectionError)?,
            ))
        })
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(())
}

#[tauri::command]
pub async fn grades_list(state: State<'_, AppStates>) -> Result<Vec<Grade>> {
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AuthenticatedState>()
        .map_err(ApplicationError::StateAquisitionError)
        .log_on_err()?;
    Ok(state
        .app_repositories
        .grades()
        .list()
        .map_err(ApplicationError::GradeListQueryError)
        .await
        .log_on_err()?)
}
