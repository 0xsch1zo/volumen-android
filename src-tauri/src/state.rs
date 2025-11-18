use std::{
    any::{self, Any},
    fmt::Debug,
};

use crate::{
    error::{ApplicationError, StatefulError},
    net::{
        synergia_api::{self, account_selector::AccountSelector},
        SynergiaApi,
    },
};

pub trait AppState: Debug + Any + Send + Sync + 'static {
    // Necessary for the compiler to allow us to transform trait to any thanks to dynamic dispatch
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct UnauthenticatedState {
    pub synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>,
}

impl UnauthenticatedState {
    pub fn new(synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>) -> Self {
        Self { synergia_api }
    }
}

#[derive(Debug)]
pub struct AccountSelctionState {
    pub account_selector: AccountSelector,
}

impl AccountSelctionState {
    pub fn new(account_selector: AccountSelector) -> Self {
        Self { account_selector }
    }
}

#[derive(Debug)]
pub struct AuthenticatedState {
    pub synergia_api: SynergiaApi<synergia_api::AuthenticatedState>,
}

impl AuthenticatedState {
    pub fn new(synergia_api: SynergiaApi<synergia_api::AuthenticatedState>) -> Self {
        Self { synergia_api }
    }
}

impl AppState for UnauthenticatedState {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl AppState for AccountSelctionState {
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
        let state = Box::new(UnauthenticatedState {
            synergia_api: SynergiaApi::try_new()?,
        });
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
