use std::cell::LazyCell;

use url::Url;

pub use crate::net::synergia_api::states::{
    authenticated::{self, Error as AuthenticatedSynergiaApiError},
    unauthenticated::Error as UnauthenticatedSynergiaApiError,
    ApiState, AuthenticatedState, UnauthenticatedState,
};

pub mod account_selector;
mod api;
mod authenticators;
mod clients;
pub mod credential_manager;
pub mod states;

const PORTAL_URL: LazyCell<Url> = LazyCell::new(|| Url::parse("https://portal.librus.pl").unwrap());

const SYNERGIA_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://synergia.librus.pl").unwrap());

const LIBRUS_API_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://api.librus.pl").unwrap());

const MESSAGES_URL: LazyCell<Url> =
    LazyCell::new(|| Url::parse("https://wiadomosci.librus.pl").unwrap());

#[derive(Debug)]
pub struct SynergiaApi<S: ApiState = UnauthenticatedState> {
    state: S,
}
