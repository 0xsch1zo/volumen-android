use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use thiserror::Error;

use tokio::sync::{broadcast, watch, RwLock};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FlightState {
    Ground,
    Flying,
}

#[derive(Debug)]
pub struct SingleParallelFlight<T: Send + Sync + Clone> {
    tx: broadcast::Sender<T>,
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
#[error("failed to recieve new state")]
pub struct StateReceiveError(#[from] watch::error::RecvError);

#[derive(Debug, Clone, Error)]
#[error("failed to update state")]
pub struct StateUpdateError<T>(#[from] watch::error::SendError<T>);

#[derive(Debug, Clone)]
pub struct StateReceiver<T: Clone>(watch::Receiver<T>);

impl<T: Clone> StateReceiver<T> {
    pub async fn recv(&mut self) -> Result<T, StateReceiveError> {
        self.0.changed().await?;
        Ok(self.0.borrow_and_update().clone())
    }
}

#[derive(Debug)]
pub struct StateWatch<T: Clone> {
    rx: watch::Receiver<T>,
    tx: watch::Sender<T>,
}

impl<T: Clone> StateWatch<T> {
    pub fn new(initial_state: T) -> Self {
        let (tx, rx) = watch::channel(initial_state);
        Self { tx, rx }
    }

    pub fn update(&self, new_state: T) -> Result<(), StateUpdateError<T>> {
        self.tx.send(new_state)?;
        Ok(())
    }

    pub fn receiver(&self) -> StateReceiver<T> {
        StateReceiver(self.rx.clone())
    }
}
