use std::error::Error as StdError;

use log::error;
use serde::Serialize;
use thiserror::Error;

use crate::{
    repositories,
    state::{self, StateTransitionError},
};

// TODO: I need better errors here
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("failed to list synergia accounts")]
    AccountListQueryError(#[source] repositories::account_selection::Error),
    #[error("failed to list grades")]
    GradeListQueryError(#[source] repositories::grades::Error),
    #[error("state aquisition failed")]
    StateAquisitionError(#[source] state::Error),
    #[error("state transition failed")]
    StateTransitionError(#[from] StateTransitionError),
}

trait ErrorChainExt: StdError {
    fn to_display_chain(&self) -> String {
        fn display(error: &dyn StdError, chain: String) -> String {
            let Some(source) = error.source() else {
                return chain;
            };
            display(source, format!("{chain}\nCaused by: {source}"))
        }
        display(&self, format!("Error: {self}"))
    }
}

impl<E: StdError> ErrorChainExt for E {}

pub trait ApplicationResultExt<T> {
    fn into_app_result(self) -> Result<T, ApplicationError>;
}

impl<T, E: Into<ApplicationError>> ApplicationResultExt<T> for Result<T, E> {
    fn into_app_result(self) -> Result<T, ApplicationError> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => Err(e.into()),
        }
    }
}

#[repr(transparent)]
pub struct LoggedApplicationError(ApplicationError);

pub trait LoggedApplicationResultExt<T> {
    fn log_on_err(self) -> Result<T, LoggedApplicationError>;
}

impl<T> LoggedApplicationResultExt<T> for Result<T, ApplicationError> {
    fn log_on_err(self) -> Result<T, LoggedApplicationError> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => {
                error!("{}", e.to_display_chain());
                Err(LoggedApplicationError(e))
            }
        }
    }
}

#[derive(Serialize)]
pub struct FrontendError {
    message: String,
    code: String,
}

impl From<LoggedApplicationError> for FrontendError {
    fn from(value: LoggedApplicationError) -> Self {
        Self {
            message: value.0.to_string(),
            code: format!("{:?}", value.0),
        }
    }
}

pub struct StatefulError<S, E> {
    pub error: E,
    pub state: S,
}

pub trait IntoStatefulErrorExt<S: Sized, E>: Sized {
    fn into_stateful_err(self, state: S) -> StatefulError<S, E>;
}

impl<S, E, I: Into<E>> IntoStatefulErrorExt<S, E> for I {
    fn into_stateful_err(self, state: S) -> StatefulError<S, E> {
        StatefulError {
            error: self.into(),
            state,
        }
    }
}

pub trait StatefulResultExt<T, S: Sized, E>: Sized {
    fn map_err_state<F>(
        self,
        transformer: impl FnOnce(S) -> F,
    ) -> std::result::Result<T, StatefulError<F, E>>;

    fn map_stateful_err<F>(
        self,
        transformer: impl FnOnce(E) -> F,
    ) -> std::result::Result<T, StatefulError<S, F>>;
}

impl<T, S: Sized, E> StatefulResultExt<T, S, E> for std::result::Result<T, StatefulError<S, E>> {
    fn map_err_state<F>(
        self,
        transformer: impl FnOnce(S) -> F,
    ) -> std::result::Result<T, StatefulError<F, E>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(StatefulError {
                error: e.error,
                state: transformer(e.state),
            }),
        }
    }

    fn map_stateful_err<F>(
        self,
        transformer: impl FnOnce(E) -> F,
    ) -> std::result::Result<T, StatefulError<S, F>> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(StatefulError {
                error: transformer(e.error),
                state: e.state,
            }),
        }
    }
}

// FIXME: the name of this macro is atrocious, change it, or even better get rid of this dogshit in
// the first place
#[macro_export]
macro_rules! stateful_result {
    ($state:expr => $res:expr) => {
        match $res {
            Ok(t) => t,
            Err(e) => {
                return Err(
                    <_ as crate::error::IntoStatefulErrorExt<_, _>>::into_stateful_err(e, $state),
                )
            }
        }
    };
}
