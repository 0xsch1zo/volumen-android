use std::{
    any::{self, Any, TypeId},
    env,
    fmt::Debug,
};

use log::LevelFilter;

use crate::{
    application_error::{
        ApplicationError, ApplicationResultExt, FrontendError, LoggedApplicationResultExt,
    },
    net::{synergia_api, SynergiaApi},
};

mod application_error;
mod common;
mod net;

#[cfg(debug_assertions)]
const fn is_debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
const fn is_debug() -> bool {
    false
}

type Result<T, E = FrontendError> = std::result::Result<T, E>;

trait AppState: Debug + Any + 'static {}

#[derive(Debug)]
struct UnauthenticatedState {
    synergia_api: SynergiaApi<synergia_api::UnauthenticatedState>,
}

#[derive(Debug)]
struct AuthenticatedState {
    synergia_api: SynergiaApi<synergia_api::AuthenticatedState>,
}

impl AppState for UnauthenticatedState {}
impl AppState for AuthenticatedState {}

struct AppStates(Option<Box<dyn AppState>>);

impl AppStates {
    async fn try_new() -> Result<Self> {
        let state = Box::new(UnauthenticatedState {
            synergia_api: SynergiaApi::with_authorized()
                .await
                .into_app_result()
                .log_on_err()?,
        });
        Ok(Self(Some(state)))
    }

    fn as_state<S: AppState>(&self) -> Result<&S, ApplicationError> {
        let state = self.0.as_ref().unwrap() as &dyn Any;
        Ok(state
            .downcast_ref()
            .ok_or(ApplicationError::StateTypeProjectionError(
                any::type_name::<S>().to_owned(),
            ))?)
    }

    async fn state_transition<S: AppState, T: AppState>(
        &mut self,
        transformer: impl AsyncFnOnce(S) -> Result<T, ApplicationError>,
    ) -> Result<(), ApplicationError> {
        let type_wanted = any::type_name::<S>().to_owned();

        let state = self.0.take().unwrap() as Box<dyn Any>;
        let state = *state
            .downcast::<S>()
            .map_err(|_| ApplicationError::StateTypeProjectionError(type_wanted))?;

        self.0 = Some(Box::new(transformer(state).await?));
        Ok(())
    }
}

#[tauri::command]
async fn send(login: String, password: String) -> Result<String> {
    SynergiaApi::with_authorized()
        .await
        .into_app_result()
        .log_on_err()?
        .login(&login, &password)
        .await
        .into_app_result()
        .log_on_err()?;
    Ok(login)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut logger = env_logger::builder();
    match env::var("RUST_LOG") {
        Err(env::VarError::NotPresent) => logger.filter_level(LevelFilter::Debug),
        _ => &mut logger,
    }
    .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![send])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
