use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;

use crate::{
    error::StatefulResultExt,
    net::{
        self,
        synergia_api::{
            self,
            api::auth::{PortalTokenPair, SynergiaAccounts},
            AuthenticatedState, StatefulError, PORTAL_URL,
        },
        ErrorStatusMiddleware, SynergiaApi,
    },
    repositories::account_selection::{SynergiaAccounts as ModelSynergiaAccounts, SynergiaUserId},
};

#[derive(Error, Debug)]
#[error("failed to construct account selector")]
pub struct AccountSelectorConstructionError(#[source] reqwest::Error);

#[derive(Error, Debug)]
pub enum AccountSelectorError {
    #[error("failed to initialize the authenticated synergia api")]
    AuthedSynergiaApiInit(#[source] synergia_api::Error),
    #[error("failed to send request grabbing account list")]
    AccountListRequestSendError(#[source] reqwest_middleware::Error),
    #[error("failed to deserialize synergia accounts")]
    SynergiaAccountsDeserializationError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct AccountSelector {
    portal_creds: PortalTokenPair,
    client: ClientWithMiddleware,
}

impl AccountSelector {
    pub fn try_new(
        portal_creds: PortalTokenPair,
    ) -> Result<Self, AccountSelectorConstructionError> {
        let client = ClientBuilder::new(
            net::default_client_options()
                .build()
                .map_err(AccountSelectorConstructionError)?,
        )
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
    ) -> Result<SynergiaApi<AuthenticatedState>, StatefulError<Self, AccountSelectorError>> {
        SynergiaApi::<AuthenticatedState>::init(id.into(), self.portal_creds)
            .await
            .map_err_state(|(_, portal_creds)| Self {
                portal_creds,
                client: self.client,
            })
            .map_stateful_err(AccountSelectorError::AuthedSynergiaApiInit)
    }

    pub async fn accounts(&self) -> Result<ModelSynergiaAccounts, AccountSelectorError> {
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
            .await
            .map_err(AccountSelectorError::AccountListRequestSendError)?
            .json::<SynergiaAccounts>()
            .await
            .map_err(AccountSelectorError::SynergiaAccountsDeserializationError)?;
        Ok(accounts.into())
    }
}
