use reqwest::{Client, StatusCode};
use thiserror::Error;

use crate::{
    error::{StatefulError, StatefulResultExt},
    net::{
        self,
        synergia_api::{
            api::auth::{PortalTokenPair, SynergiaAccounts},
            states::{authenticated, AuthenticatedState},
            PORTAL_URL,
        },
        SynergiaApi,
    },
    repositories::account_selection::{SynergiaAccounts as ModelSynergiaAccounts, SynergiaUserId},
    sync::{LogoutEventEmissionError, LogoutSignaler},
};

#[derive(Error, Debug)]
#[error("failed to construct account selector")]
pub struct AccountSelectorConstructionError(#[source] reqwest::Error);

#[derive(Error, Debug)]
pub enum AccountSelectorError {
    #[error("failed to initialize the authenticated synergia api")]
    AuthedSynergiaApiInit(#[source] authenticated::Error),
    #[error("failed to send request grabbing account list")]
    AccountListRequestSendError(#[source] reqwest::Error),
    #[error("request for account list returned with error status code")]
    AccountListRequestErrorStatus(#[source] reqwest::Error),
    #[error("failed to deserialize synergia accounts")]
    SynergiaAccountsDeserializationError(#[from] reqwest::Error),
    #[error(transparent)]
    LogoutEventEmissionError(#[from] LogoutEventEmissionError),
}

#[derive(Debug)]
pub struct AccountSelector {
    portal_creds: PortalTokenPair,
    client: Client,
}

impl AccountSelector {
    pub fn try_new(
        portal_creds: PortalTokenPair,
    ) -> Result<Self, AccountSelectorConstructionError> {
        let client = net::default_client_options()
            .build()
            .map_err(AccountSelectorConstructionError)?;
        Ok(Self {
            portal_creds,
            client,
        })
    }

    pub async fn select(
        self,
        id: SynergiaUserId,
        logout_signaler: LogoutSignaler,
    ) -> Result<SynergiaApi<AuthenticatedState>, StatefulError<Self, AccountSelectorError>> {
        SynergiaApi::<AuthenticatedState>::init(id.into(), self.portal_creds, logout_signaler)
            .await
            .map_err_state(|(_, portal_creds)| Self {
                portal_creds,
                client: self.client,
            })
            .map_stateful_err(AccountSelectorError::AuthedSynergiaApiInit)
    }

    pub async fn accounts(
        &self,
        logout_signaler: &LogoutSignaler,
    ) -> Result<ModelSynergiaAccounts, AccountSelectorError> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";
        let portal_access_token = &self.portal_creds.access_token;

        let resp = self
            .client
            .get(PORTAL_URL.join(SYNERGIA_ACCOUNT_ENDPOINT).unwrap())
            .bearer_auth(portal_access_token.as_inner())
            .send()
            .await
            .map_err(AccountSelectorError::AccountListRequestSendError)?;
        if resp.status() == StatusCode::UNAUTHORIZED {
            logout_signaler.send_logout_event()?;
        }
        resp.error_for_status_ref()
            .map_err(AccountSelectorError::AccountListRequestErrorStatus)?;

        let accounts = resp
            .json::<SynergiaAccounts>()
            .await
            .map_err(AccountSelectorError::SynergiaAccountsDeserializationError)?;
        Ok(accounts.into())
    }
}
