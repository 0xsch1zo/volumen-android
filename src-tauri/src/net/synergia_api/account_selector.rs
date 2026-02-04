use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;

use crate::{
    error::StatefulResultExt,
    net::{
        self,
        synergia_api::{
            api::auth::{PortalTokenPair, SynergiaAccounts},
            AuthenticatedState, StatefulError, PORTAL_URL,
        },
        ErrorStatusMiddleware, SynergiaApi,
    },
    repositories::account_selection::{SynergiaAccounts as ModelSynergiaAccounts, SynergiaUserId},
};

#[derive(Error, Debug)]
pub enum AccountSelectorError {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("reqwest error")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
}

type Result<T, E = AccountSelectorError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct AccountSelector {
    portal_creds: PortalTokenPair,
    client: ClientWithMiddleware,
}

impl AccountSelector {
    pub fn try_new(portal_creds: PortalTokenPair) -> Result<Self> {
        let client = ClientBuilder::new(net::default_client_options().build()?)
            .with(ErrorStatusMiddleware)
            .build();
        Ok(Self {
            portal_creds,
            client,
        })
    }

    pub async fn select(
        self,
        id: SynergiaUserId,
    ) -> Result<SynergiaApi<AuthenticatedState>, StatefulError<Self>> {
        SynergiaApi::<AuthenticatedState>::init(id.into(), self.portal_creds)
            .await
            .map_err_state(|(_, portal_creds)| Self {
                portal_creds,
                client: self.client,
            })
    }

    pub async fn accounts(&self) -> Result<ModelSynergiaAccounts> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";
        let portal_access_token = &self.portal_creds.access_token;

        let client = net::default_client_options().build()?;
        let client = ClientBuilder::new(client)
            .with(ErrorStatusMiddleware)
            .build();

        let accounts = client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .await?
            .json::<SynergiaAccounts>()
            .await?;
        Ok(accounts.into())
    }
}
