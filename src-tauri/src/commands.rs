use futures::TryFutureExt;
use tauri::{AppHandle, Manager, State};

use crate::{
    domain::daily_timetable::DailyTimetable,
    error::{
        ApplicationError, ApplicationResultExt, LoggedApplicationResultExt, StatefulResultExt,
    },
    repositories::{account_selection::SynergiaAccount, grades::Grade},
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
pub async fn select_account(app_handle: AppHandle, account: SynergiaAccount) -> Result<()> {
    let state = app_handle.state::<AppStates>();
    let mut state_lock = state.lock().await;
    state_lock
        .state_transition::<AccountSelectionState, AuthenticatedState>(async |s| {
            Ok(AuthenticatedState::new(
                s.account_selection_repo
                    .select(account, app_handle.clone())
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

#[tauri::command]
pub async fn current_account(state: State<'_, AppStates>) -> Result<SynergiaAccount> {
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AuthenticatedState>()
        .map_err(ApplicationError::StateAquisitionError)
        .log_on_err()?;
    Ok(state.app_repositories.session().current_account())
}

#[tauri::command]
pub async fn daily_timetable(state: State<'_, AppStates>) -> Result<DailyTimetable> {
    let state_lock = state.lock().await;
    let state = state_lock
        .as_state::<AuthenticatedState>()
        .map_err(ApplicationError::StateAquisitionError)
        .log_on_err()?;

    Ok(state
        .app_usecases
        .daily_timetable()
        .await
        .map_err(ApplicationError::DailyTimetableQueryError)
        .log_on_err()?)
}
