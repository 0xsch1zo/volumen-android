use std::sync::Arc;

use crate::{
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        account_selection::SynergiaAccount, calendar::CalendarRepository, events::EventsRepository,
        grades::GradesRepository, messages::MessagesRepository, session::SessionRepository,
        timetable::TimetableRepository,
    },
};

pub mod account_selection;
pub mod calendar;
pub mod events;
pub mod grades;
pub mod login;
pub mod me;
pub mod messages;
pub mod session;
pub mod subjects;
pub mod timetable;
pub mod users;

pub use account_selection::AccountSelectionRepository;
pub use login::LoginRepository;
use tauri::AppHandle;

#[derive(Clone, Debug)]
pub struct AppRepositories {
    grades: GradesRepository,
    messages: MessagesRepository,
    timetables: TimetableRepository,
    events: EventsRepository,
    session: SessionRepository,
    calendar: CalendarRepository,
}

impl AppRepositories {
    pub fn new(
        synergia_api: SynergiaApi<AuthenticatedState>,
        account: SynergiaAccount,
        app_handle: AppHandle,
    ) -> Self {
        let synergia_api = Arc::new(synergia_api);

        let grades = GradesRepository::new(Arc::clone(&synergia_api));
        let messages = MessagesRepository::new(Arc::clone(&synergia_api));
        let timetables = TimetableRepository::new(Arc::clone(&synergia_api));
        let events = EventsRepository::new(Arc::clone(&synergia_api));
        let session = SessionRepository::new(Arc::clone(&synergia_api), account, app_handle);
        let calendar = CalendarRepository::new(Arc::clone(&synergia_api));

        Self {
            grades,
            messages,
            timetables,
            events,
            session,
            calendar,
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

    #[allow(unused)]
    pub fn session(&self) -> &SessionRepository {
        &self.session
    }
}
