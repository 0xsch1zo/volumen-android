use std::{
    any::{self, Any},
    fmt::Debug,
};

use crate::{
    error::{ApplicationError, StatefulError},
    repositories::{AccountSelectionRepository, LoginRepository, MainRepo},
};

pub trait AppState: Debug + Any + Send + Sync + 'static {
    // Necessary for the compiler to allow us to transform trait to any thanks to dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct UnauthenticatedState {
    pub login_repo: LoginRepository,
}

impl UnauthenticatedState {
    pub fn new(login_repo: LoginRepository) -> Self {
        Self { login_repo }
    }
}

#[derive(Debug)]
pub struct AccountSelectionState {
    pub account_selection_repo: AccountSelectionRepository,
}

impl AccountSelectionState {
    pub fn new(account_selection_repo: AccountSelectionRepository) -> Self {
        Self {
            account_selection_repo,
        }
    }
}

#[derive(Debug)]
pub struct AuthenticatedState {
    pub main_repo: MainRepo,
}

impl AuthenticatedState {
    pub fn new(main_repo: MainRepo) -> Self {
        Self { main_repo }
    }
}

impl AppState for UnauthenticatedState {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AppState for AccountSelectionState {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AppState for AuthenticatedState {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct AppStatesInner(Option<Box<dyn AppState>>);

impl AppStatesInner {
    pub fn try_new() -> Result<Self, ApplicationError> {
        let state = Box::new(UnauthenticatedState::new(LoginRepository::try_new()?));
        Ok(Self(Some(state)))
    }

    pub fn as_state<S: AppState>(&self) -> Result<&S, ApplicationError> {
        let state = self.0.as_ref().unwrap();

        Ok(state
            .as_any()
            .downcast_ref()
            .ok_or(ApplicationError::WrongState(
                any::type_name::<S>().to_owned(),
            ))?)
    }

    pub async fn state_transition<S: AppState, T: AppState>(
        &mut self,
        transformer: impl AsyncFnOnce(S) -> Result<T, StatefulError<S, ApplicationError>>,
    ) -> Result<(), ApplicationError> {
        let type_wanted = any::type_name::<S>().to_owned();

        let state = self.0.take().unwrap() as Box<dyn Any>;
        let state = *state
            .downcast::<S>()
            .map_err(|_| ApplicationError::WrongState(type_wanted))?;

        match transformer(state).await {
            Ok(s) => {
                self.0 = Some(Box::new(s));
            }
            Err(e) => {
                self.0 = Some(Box::new(e.state));
                return Err(e.error);
            }
        };
        Ok(())
    }
}

pub type AppStates = tauri::async_runtime::Mutex<AppStatesInner>;
