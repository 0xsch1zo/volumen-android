use tokio::sync::{
    broadcast::{self, Sender},
    RwLock,
};

#[derive(Clone, Copy)]
enum FlightState {
    Ground,
    Flying,
}

struct SingleParallelFlight<T: Send + Sync + Clone> {
    tx: Sender<T>,
    state: RwLock<FlightState>,
}

impl<T: Send + Sync + Clone> SingleParallelFlight<T> {
    fn new() -> Self {
        let (tx, _) = broadcast::channel::<T>(1);
        Self {
            tx,
            state: RwLock::new(FlightState::Ground),
        }
    }

    async fn work(&self, work: impl AsyncFnOnce() -> T) -> T {
        match *self.state.read().await {
            FlightState::Ground => {
                *self.state.write().await = FlightState::Flying;
                let t = work().await;
                self.tx.send(t.clone());
                *self.state.write().await = FlightState::Ground;
                t
            }
            FlightState::Flying => self.tx.subscribe().recv().await.unwrap(),
        }
    }
}
