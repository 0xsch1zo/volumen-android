use std::sync::Arc;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use thiserror::Error;

use crate::net::{
    self,
    synergia_api::{
        authenticators::{MainAuthenticator, MessagesAuthenticator, MessagesAuthenticatorError},
        states::unauthenticated::UnauthenticatedRedirectPolicy,
    },
    ErrorStatusMiddleware,
};

#[derive(Error, Debug)]
#[error("failed to construct reqwest client")]
pub struct UnauthenticatedClientConstructionError(#[from] reqwest::Error);

#[derive(Error, Debug)]
#[error("failed to construct reqwest client")]
pub struct AuthenticatedClientConstructionError(#[from] reqwest::Error);

#[derive(Error, Debug)]
pub enum MessagesClientInitError {
    #[error("failed to construct reqwest client")]
    ClientConstructionError(#[source] reqwest::Error),
    #[error("failed to initialize messages authentiactor")]
    MessagesAuthenticatorInitError(#[source] MessagesAuthenticatorError),
}

#[derive(Debug)]
pub struct UnauthenticatedClient(ClientWithMiddleware);

impl UnauthenticatedClient {
    pub fn try_new(
        redirect_policy: UnauthenticatedRedirectPolicy,
    ) -> Result<Self, UnauthenticatedClientConstructionError> {
        let client = net::default_client_options()
            .redirect(redirect_policy.into_inner())
            .cookie_store(true)
            .build()?;

        Ok(Self(
            ClientBuilder::new(client)
                .with(ErrorStatusMiddleware)
                .build(),
        ))
    }

    pub fn as_inner(&self) -> &ClientWithMiddleware {
        &self.0
    }
}

#[derive(Debug)]
pub struct MainAuthenticatedClient(ClientWithMiddleware);

impl MainAuthenticatedClient {
    pub fn try_new(
        main_authenticator: Arc<MainAuthenticator>,
    ) -> Result<Self, AuthenticatedClientConstructionError> {
        let client = net::default_client_options().build()?;

        Ok(Self(
            ClientBuilder::new(client)
                .with(ErrorStatusMiddleware)
                .with_arc(main_authenticator)
                .build(),
        ))
    }

    pub fn as_inner(&self) -> &ClientWithMiddleware {
        &self.0
    }
}

#[derive(Debug)]
pub struct MessagesClient(ClientWithMiddleware);

impl MessagesClient {
    pub async fn init(
        main_authenticator: Arc<MainAuthenticator>,
    ) -> Result<Self, MessagesClientInitError> {
        let client = net::default_client_options()
            .build()
            .map_err(MessagesClientInitError::ClientConstructionError)?;

        let messages_authenticator = MessagesAuthenticator::init(main_authenticator)
            .await
            .map_err(MessagesClientInitError::MessagesAuthenticatorInitError)?;
        Ok(Self(
            ClientBuilder::new(client)
                .with(ErrorStatusMiddleware)
                .with(messages_authenticator)
                .build(),
        ))
    }

    pub fn as_inner(&self) -> &ClientWithMiddleware {
        &self.0
    }
}
