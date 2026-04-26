use serde::Deserialize;

use crate::{
    net::synergia_api::api::Reference,
    repositories::{self, calendar as models},
};

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(from = "Reference")]
pub struct EventId(usize);

impl From<Reference> for EventId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<EventId> for repositories::events::EventId {
    fn from(value: EventId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Calendar {
    #[serde(rename = "HomeWorks")]
    events: Vec<EventId>,
}

impl From<Calendar> for models::Calendar {
    fn from(value: Calendar) -> Self {
        Self {
            events: value.events.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CalendarResponse {
    pub calendar: Calendar,
}
