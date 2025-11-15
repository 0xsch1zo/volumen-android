use std::error::Error as StdError;

use log::error;
use serde::Serialize;
use thiserror::Error;

use crate::net::{synergia_api::account_management::AccountManagerError, SynergiaApiError};

#[derive(Error, Debug)]
pub enum ApplicationError {
    // shouldn't be handled here really
    #[error("synergia api error occured")]
    SynergiaApiError(#[from] SynergiaApiError),
    #[error("account manager error")]
    AccountManagerError(#[from] AccountManagerError),
    #[error("wanted to aquire wrong state: {0}")]
    WrongState(String),
}

trait ErrorChainExt: StdError {
    fn to_display_chain(&self) -> String {
        fn display(error: &(dyn StdError), chain: String) -> String {
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
