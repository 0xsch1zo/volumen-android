use thiserror::Error;

use crate::{
    error,
    net::{synergia_api::account_selector::AccountSelectorError, SynergiaApiError},
    repositories::users::UserId,
};

pub mod account_selection;
pub mod grades;
pub mod login;
pub mod main;
pub mod messages;
pub mod subjects;
pub mod users;

pub use account_selection::AccountSelectionRepository;
pub use login::LoginRepository;
pub use main::MainRepository;

// FIXME: this error handling is ass
#[derive(Error, Debug)]
pub enum Error {
    #[error("syneriga api error")]
    SynerigaApiError(#[from] SynergiaApiError),
    #[error("account selector error")]
    AccountSelectorError(#[from] AccountSelectorError),
    #[error("user repository error")]
    UserRepositoryError(UserId),
    #[error("grade repo error")]
    GradeRepoError(#[from] grades::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

type StatefulError<S, E = Error> = error::StatefulError<S, E>;
