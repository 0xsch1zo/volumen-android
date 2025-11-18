use thiserror::Error;

use crate::{
    error,
    net::{synergia_api::account_selector::AccountSelectorError, SynergiaApiError},
};

pub mod account_selection;
pub mod entities;
pub mod grades;
pub mod login;
pub mod main;

pub use account_selection::AccountSelectionRepository;
pub use login::LoginRepository;
pub use main::MainRepo;

#[derive(Error, Debug)]
pub enum Error {
    #[error("syneriga api error")]
    SynerigaApiError(#[from] SynergiaApiError),
    #[error("account selector error")]
    AccountSelectorError(#[from] AccountSelectorError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

type StatefulError<S, E = Error> = error::StatefulError<S, E>;
