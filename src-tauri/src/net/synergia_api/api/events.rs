use serde::Deserialize;

use crate::{
    net::synergia_api::api::{subjects::SubjectId, users::UserId, Reference},
    repositories::events as models,
};

#[derive(Deserialize)]
#[serde(from = "Reference")]
struct EventId(usize);

impl From<Reference> for EventId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<EventId> for models::EventId {
    fn from(value: EventId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Deserialize)]
#[serde(from = "Reference")]
pub struct CategoryId(usize);

impl From<Reference> for CategoryId {
    fn from(value: Reference) -> Self {
        Self(value.into_id())
    }
}

impl From<CategoryId> for models::CategoryId {
    fn from(value: CategoryId) -> Self {
        Self::new(value.0)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Category {
    id: CategoryId,
    name: String,
}

impl From<Category> for models::Category {
    fn from(value: Category) -> Self {
        Self {
            id: value.id.into(),
            name: value.name,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CategoriesResponse {
    pub categories: Vec<Category>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShallowEvent {
    id: EventId,
    content: String,
    date: String,
    category: CategoryId,
    time_from: String,
    time_to: String,
    created_by: UserId,
    subject: Option<SubjectId>,
    add_date: String,
}

impl From<ShallowEvent> for models::ShallowEvent {
    fn from(value: ShallowEvent) -> Self {
        Self {
            id: value.id.into(),
            content: value.content,
            date: value.date,
            category: value.category.into(),
            time_from: value.time_from,
            time_to: value.time_to,
            created_by: value.created_by.into(),
            subject: value.subject.map(Into::into),
            add_date: value.add_date,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventsResponse {
    #[serde(rename = "HomeWorks")]
    pub events: Vec<ShallowEvent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EventResponse {
    #[serde(rename = "HomeWork")]
    pub event: ShallowEvent,
}
