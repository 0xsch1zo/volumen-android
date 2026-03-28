use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use thiserror::Error;
use tokio::sync::{
    broadcast::{self, Sender},
    RwLock,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FlightState {
    Ground,
    Flying,
}

#[derive(Debug)]
pub struct SingleParallelFlight<T: Send + Sync + Clone> {
    tx: Sender<T>,
    state: RwLock<FlightState>,
}

impl<T: Send + Sync + Clone> SingleParallelFlight<T> {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel::<T>(1);
        Self {
            tx,
            state: RwLock::new(FlightState::Ground),
        }
    }

    pub async fn work(&self, work: impl AsyncFnOnce() -> T) -> T {
        let state = *self.state.read().await;
        match state {
            FlightState::Ground => {
                *self.state.write().await = FlightState::Flying;
                let t = work().await;
                // IMPORATNT: this fails only if there are no listeners, in that case we don't
                // really care anyway, because it just means that there weren't any concurrent
                // calls so we can just function normally
                let _ = self.tx.send(t.clone());
                *self.state.write().await = FlightState::Ground;
                t
            }
            FlightState::Flying => self.tx.subscribe().recv().await.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Error)]
#[error("failed to emit logout event")]
pub struct LogoutEventEmissionError(Arc<tauri::Error>);

#[derive(Debug, Clone)]
pub struct LogoutSignaler {
    app_handle: AppHandle,
}

impl LogoutSignaler {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn send_logout_event(&self) -> Result<(), LogoutEventEmissionError> {
        self.app_handle
            .emit("logout", ())
            .map_err(Arc::new)
            .map_err(LogoutEventEmissionError)
    }
}
