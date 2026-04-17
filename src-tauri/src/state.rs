use std::{
    any::{self, Any},
    fmt::Debug,
};

use thiserror::Error;

use crate::{
    error::StatefulError,
    repositories::{self, login, AccountSelectionRepository, AppRepositories, LoginRepository},
};

#[derive(Error, Debug)]
pub enum StateTransitionError {
    #[error("encountered error while logging in")]
    LoginError(#[source] login::Error),
    #[error("account selection error")]
    AcccountSelectionError(#[source] repositories::account_selection::Error),
    #[error("wanted to aquire wrong state: {0}")]
    WrongState(String),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("wanted to aquire wrong state: {0}")]
    WrongState(String),
    #[error("initial state construction error")]
    InitialStateConstructionError(#[source] repositories::login::Error),
}

pub trait AppState: Debug + Any + Send + Sync + 'static {
    // Necessary for the compiler to allow us to transform trait to any thanks to dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct UnauthenticatedState {
    pub login_repo: LoginRepository,
}

impl UnauthenticatedState {
    pub fn try_new() -> Result<Self, Error> {
        Ok(Self {
            login_repo: LoginRepository::try_new().map_err(Error::InitialStateConstructionError)?,
        })
    }

    pub fn from_repo(login_repo: LoginRepository) -> Self {
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
    pub app_repositories: AppRepositories,
}

impl AuthenticatedState {
    pub fn new(app_repositories: AppRepositories) -> Self {
        Self { app_repositories }
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
    pub fn try_new() -> Result<Self, Error> {
        let state = Box::new(UnauthenticatedState::try_new()?);
        Ok(Self(Some(state)))
    }

    pub fn as_state<S: AppState>(&self) -> Result<&S, Error> {
        let state = self.0.as_ref().unwrap();

        Ok(state
            .as_any()
            .downcast_ref()
            .ok_or(Error::WrongState(any::type_name::<S>().to_owned()))?)
    }

    pub async fn state_transition<S, T>(
        &mut self,
        transformer: impl AsyncFnOnce(S) -> Result<T, StatefulError<S, StateTransitionError>>,
    ) -> Result<(), StateTransitionError>
    where
        S: AppState,
        T: AppState,
    {
        let type_wanted = any::type_name::<S>().to_owned();

        let state = self.0.take().unwrap() as Box<dyn Any>;
        let state = *state
            .downcast::<S>()
            .map_err(|_| StateTransitionError::WrongState(type_wanted))?;

        match transformer(state).await {
            Ok(s) => {
                self.0 = Some(Box::new(s));
            }
            Err(e) => {
                self.0 = Some(Box::new(e.state));
                return Err(e.error)?;
            }
        };
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        let state = Box::new(UnauthenticatedState::try_new()?);
        self.0 = Some(state);
        Ok(())
    }
}

pub type AppStates = tauri::async_runtime::Mutex<AppStatesInner>;
