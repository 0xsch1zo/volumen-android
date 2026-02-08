pub mod authenticated;
pub mod unauthenticated;

pub use authenticated::AuthenticatedState;
pub use unauthenticated::UnauthenticatedState;

pub trait ApiState {}

impl ApiState for UnauthenticatedState {}
impl ApiState for AuthenticatedState {}

