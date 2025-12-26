use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;

use crate::{
    error::StatefulResultExt,
    net::{
        self,
        synergia_api::{
            api::auth::{SynergiaAccounts, Tokens},
            auth_manager::AuthorizationManager,
            token_management::TokensApi,
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
    tokens: Tokens,
    tokens_api: TokensApi,
    client: ClientWithMiddleware,
}

impl AccountSelector {
    pub fn try_new(tokens: Tokens, tokens_api: TokensApi) -> Result<Self> {
        let client = net::default_client_options().build()?;
        let client = ClientBuilder::new(client)
            .with(ErrorStatusMiddleware)
            .build();
        Ok(Self {
            tokens,
            tokens_api,
            client,
        })
    }

    pub fn select(
        self,
        id: SynergiaUserId,
    ) -> Result<SynergiaApi<AuthenticatedState>, StatefulError<Self>> {
        let auth_manager = AuthorizationManager::new(self.tokens, self.tokens_api, id.into());
        SynergiaApi::<AuthenticatedState>::try_from_auth_manager(auth_manager)
            .map_err_state(AuthorizationManager::decay)
            .map_err_state(|(tokens, tokens_api)| Self {
                tokens,
                tokens_api,
                client: self.client,
            })
    }

    pub async fn accounts(&self) -> Result<ModelSynergiaAccounts> {
        const SYNERGIA_ACCOUNT_ENDPOINT: &str = "/api/v3/SynergiaAccounts";
        let portal_access_token = &self.tokens.portal_token_pair.access_token;

        let client = Client::builder().connection_verbose(true).build()?;
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
