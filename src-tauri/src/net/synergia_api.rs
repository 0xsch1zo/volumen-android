use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::net::{self, ResponseExt};

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("url parsing error")]
    UrlParsingError(#[from] url::ParseError),
    #[error("response error")]
    ResponseError(#[from] net::ResponseError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    action: String,
    login: String,
    pass: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    status: String,
    go_to: String,
}

pub trait ApiState {}

pub struct UnauthorizedState {
    client: Client,
    base_url: Url,
}

pub struct LoggedInState {
    client: Client,
    base_url: Url,
}

impl ApiState for UnauthorizedState {}
impl ApiState for LoggedInState {}

pub struct SynergiaApi<S: ApiState> {
    state: S,
}

impl SynergiaApi<UnauthorizedState> {
    pub fn try_new() -> Result<Self> {
        const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

        Ok(Self {
            state: UnauthorizedState {
                client: Client::builder()
                    .cookie_store(true)
                    .user_agent(USER_AGENT)
                    .build()?,
                base_url: Url::parse("api.librus.pl").unwrap(),
            },
        })
    }

    pub async fn login(self, login: &str, pass: &str) -> Result<SynergiaApi<LoggedInState>> {
        const AUTH_ENPOINT: &str = "/OAuth/Authorization?client_id=46"; // why 46 you may ask, ...
                                                                        // I don't know
        let resp = self
            .state
            .client
            .post(self.state.base_url.join(AUTH_ENPOINT).unwrap())
            .query(&LoginRequest {
                action: "login".to_owned(),
                login: login.to_owned(),
                pass: pass.to_owned(),
            })
            .send()
            .await?
            .error_on_status()
            .await?
            .json::<LoginResponse>()
            .await?;

        self.state
            .client
            .get(self.state.base_url.join(&resp.go_to)?)
            .send()
            .await?;

        Ok(SynergiaApi {
            state: LoggedInState {
                client: self.state.client,
                base_url: self.state.base_url,
            },
        })
    }
}
