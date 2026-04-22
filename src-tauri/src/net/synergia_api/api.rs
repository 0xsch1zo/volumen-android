use serde::{Deserialize, Serialize};

pub mod auth;
pub mod calendar;
pub mod events;
pub mod grades;
pub mod me;
pub mod messages;
pub mod subjects;
pub mod timetable;
pub mod users;

// Generic reference used for internal purposes
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
#[serde(untagged)]
enum Reference {
    #[serde(rename_all = "PascalCase")]
    Linked {
        id: usize,
        url: String,
    },
    Standalone(usize),
}

impl Reference {
    pub fn into_id(self) -> usize {
        match self {
            Self::Linked { id, .. } => id,
            Self::Standalone(id) => id,
        }
    }
}
