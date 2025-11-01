use std::{
    any::{self, Any},
    fmt::Debug,
};

use crate::{
    error::ApplicationError,
    net::{synergia_api, SynergiaApi},
};

pub trait AppState: Debug + Any + Send + Sync + 'static {}

pub struct StateTranstionError<S: AppState> {
    state: S,
    error: ApplicationError,
}

impl<S: AppState> StateTranstionError<S> {
    pub fn new(state: S, error: ApplicationError) -> Self {
        Self { state, error }
    }
}

#[derive(Debug)]
pub struct UnauthenticatedState {
    pub synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>,
}

#[derive(Debug)]
pub struct AuthenticatedState {
    pub synergia_api: SynergiaApi<synergia_api::AuthenticatedState>,
}

impl AppState for UnauthenticatedState {}
impl AppState for AuthenticatedState {}

pub struct AppStatesInner(Option<Box<dyn AppState>>);

impl AppStatesInner {
    pub fn try_new() -> Result<Self, ApplicationError> {
        let state = Box::new(UnauthenticatedState {
            synergia_api: SynergiaApi::try_new()?,
        });
        Ok(Self(Some(state)))
    }

    pub fn as_state<S: AppState>(&self) -> Result<&S, ApplicationError> {
        let state = self.0.as_ref().unwrap() as &dyn Any;

        Ok(state.downcast_ref().ok_or(ApplicationError::WrongState(
            any::type_name::<S>().to_owned(),
        ))?)
    }

    pub async fn state_transition<S: AppState, T: AppState>(
        &mut self,
        transformer: impl AsyncFnOnce(S) -> Result<T, StateTranstionError<S>>,
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
