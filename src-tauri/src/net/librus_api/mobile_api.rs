use itertools::Itertools;
use log::warn;
use reqwest::Client;
use scraper::{Html, Selector};
use thiserror::Error;
use url::Url;

use crate::{
    common::TakeExactlyExt,
    net::{
        self,
        librus_api::mobile_api::private_types::{LoginAttrKinds, LoginAttrs},
    },
};

mod private_types;
mod public_types;

#[derive(Error, Debug)]
pub enum Error {
    #[error("a network error occured")]
    ReqwestError(#[from] reqwest::Error),
    #[error("login attribute not found: {0:?}")]
    LoginAttrNotFound(LoginAttrKinds),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub trait ApiState {}
pub struct UnauthenticatedState {}
pub struct AuthenticatedState {}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

pub struct MobileApi<S: ApiState = UnauthenticatedState> {
    state: S,
    client: Client,
    portal_url: Url,
    api_url: Url,
}

impl MobileApi<UnauthenticatedState> {
    pub fn try_new() -> Result<Self> {
        Ok(Self {
            state: UnauthenticatedState {},
            client: net::default_client_options().build()?,
            portal_url: Url::parse("https://portal.librus.pl/").unwrap(),
            api_url: Url::parse("https://api.librus.pl/").unwrap(),
        })
    }

    pub async fn login(self, email: &str, password: &str) -> Result<()> {
        let attrs = self.fetch_login_attrs().await?;
        println!("{attrs:?}");
        Ok(())
    }

    async fn fetch_login_attrs(&self) -> Result<LoginAttrs> {
        fn scrape_attributes(html: &str) -> Result<LoginAttrs> {
            let document = Html::parse_document(html);
            let redirect_to_selector =
                Selector::parse(r#"input[type="hidden"][name="redirectTo"][value]"#).unwrap();
            let redirect_crc_selector =
                Selector::parse(r#"input[type="hidden"][name="redirectCrc"][value]"#).unwrap();
            let token = Selector::parse(r#"input[type="hidden"][name="_token"][value]"#).unwrap();

            let (redirect_to, redirect_crc, token) = [
                (redirect_to_selector, LoginAttrKinds::RedirectTo),
                (redirect_crc_selector, LoginAttrKinds::RedirectCrc),
                (token, LoginAttrKinds::Token),
            ]
            .into_iter()
            .map(|(selector, attr_type)| {
                Ok(document
                    .select(&selector)
                    .take_exactly::<Vec<_>>(1)
                    .inspect_surplus(|_| warn!("many elements matched: {attr_type:?}"))
                    .enough_or(Error::LoginAttrNotFound(attr_type))?[0]
                    .attr("value")
                    .unwrap()
                    .to_owned())
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect_tuple()
            .unwrap();

            Ok(LoginAttrs {
                redirect_to,
                redirect_crc,
                token,
            })
        }

        const ATTR_ENDPOINT: &str = "/konto-librus/redirect/dru";
        let html = self
            .client
            .get(self.portal_url.join(ATTR_ENDPOINT).unwrap())
            .send()
            .await?
            .text()
            .await?;
        scrape_attributes(&html)
    }
}
