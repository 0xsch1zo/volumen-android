use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{CacheComputeError, SingleEntryCache},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to fetch user details")]
    MeError(#[source] CacheComputeError),
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: usize,
    pub user_id: usize,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ClassId(usize);

impl ClassId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn as_inner(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Me {
    pub account: Account,
    pub class: ClassId,
}

#[derive(Debug, Clone)]
pub struct MeRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    me_cache: SingleEntryCache<Me>,
}

impl MeRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            me_cache: SingleEntryCache::new(),
        }
    }

    pub async fn me(&self) -> Result<Me, Error> {
        self.me_cache
            .try_get_with(async { self.synergia_api.fetch_me().await })
            .await
            .map_err(Error::MeError)
    }
}
