use thiserror::Error;

use crate::error;

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

#[derive(Error, Debug)]
pub enum Error {
    #[error("account selection repository error")]
    AccountSelectionRepoError(#[from] account_selection::Error),
    #[error("login repository error")]
    LoginRepoError(#[from] login::Error),
    #[error("main repository error")]
    MainRepoError(#[from] main::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

type StatefulError<S, E = Error> = error::StatefulError<S, E>;
