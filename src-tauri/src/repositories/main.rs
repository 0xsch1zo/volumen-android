use std::sync::Arc;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        events::EventsRepository, grades::GradesRepository, messages::MessagesRepository,
        timetable::TimetableRepository,
    },
};

#[derive(Debug)]
pub struct MainRepository {
    grades: GradesRepository,
    messages: MessagesRepository,
    timetables: TimetableRepository,
    events: EventsRepository,
}

impl MainRepository {
    pub fn new(synergia_api: SynergiaApi<AuthenticatedState>) -> Self {
        let synergia_api = Arc::new(synergia_api);

        let grades = GradesRepository::new(Arc::clone(&synergia_api));
        let messages = MessagesRepository::new(Arc::clone(&synergia_api));
        let timetables = TimetableRepository::new(Arc::clone(&synergia_api));
        let events = EventsRepository::new(Arc::clone(&synergia_api));

        Self {
            grades,
            messages,
            timetables,
            events,
        }
    }

    #[allow(unused)]
    pub fn grades(&self) -> &GradesRepository {
        &self.grades
    }

    #[allow(unused)]
    pub fn messages(&self) -> &MessagesRepository {
        &self.messages
    }

    #[allow(unused)]
    pub fn timetables(&self) -> &TimetableRepository {
        &self.timetables
    }

    #[allow(unused)]
    pub fn events(&self) -> &EventsRepository {
        &self.events
    }
}
