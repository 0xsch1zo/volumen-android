mod categories;

use std::sync::Arc;

use futures::{stream, StreamExt, TryFutureExt, TryStreamExt};
use itertools::Itertools;
use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{synergia_api::AuthenticatedState, SynergiaApi},
    repositories::{
        calendar::Calendar,
        events::categories::CategoriesRepository,
        subjects::{self, Subject, SubjectId, SubjectsRepository},
        users::{self, User, UserId, UsersRepository},
    },
};

pub use categories::{Category, CategoryId};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch event list")]
    EventListFetchError(#[source] CacheComputeError),
    #[error("failed to fetch event")]
    EventFetchError(#[source] CacheComputeError),
    #[error("failed to fetch category")]
    CategoryFetchError(#[source] categories::Error),
    #[error("failed to fetch user")]
    UserFetchError(#[source] users::Error),
    #[error("failed to fetch subject")]
    SubjectFetchError(#[source] subjects::Error),
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(usize);

impl EventId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct ShallowEvent {
    pub id: EventId,
    pub content: String,
    pub date: String,
    pub category: CategoryId,
    pub time_from: String,
    pub time_to: String,
    pub created_by: UserId,
    pub subject: Option<SubjectId>,
    pub add_date: String,
}

impl Keyable<EventId> for ShallowEvent {
    fn key(&self) -> EventId {
        self.id
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Event {
    pub id: EventId,
    pub content: String,
    pub date: String,
    pub category: Category,
    pub time_from: String,
    pub time_to: String,
    pub created_by: User,
    pub subject: Option<Subject>,
    pub add_date: String,
}

struct EventFactory<'a> {
    categories: &'a CategoriesRepository,
    users: &'a UsersRepository,
    subjects: &'a SubjectsRepository,
}

impl<'a> EventFactory<'a> {
    fn new(
        categories: &'a CategoriesRepository,
        users: &'a UsersRepository,
        subjects: &'a SubjectsRepository,
    ) -> Self {
        Self {
            categories,
            users,
            subjects,
        }
    }

    async fn create_from_shallow(&self, shallow: ShallowEvent) -> Result<Event, Error> {
        let category_fut = self
            .categories
            .category(shallow.category)
            .map_err(Error::CategoryFetchError);
        let created_by_fut = self
            .users
            .user(shallow.created_by)
            .map_err(Error::UserFetchError);
        let subject_fut = shallow
            .subject
            .map(|s| self.subjects.subject(s).map_err(Error::SubjectFetchError));
        if let Some(subject_fut) = subject_fut {
            let (category, created_by, subject) =
                // TODO: not parallel fix this using spawn
                tokio::try_join!(category_fut, created_by_fut, subject_fut)?;
            Ok(Event {
                category,
                created_by,
                subject: Some(subject),
                time_from: shallow.time_from,
                time_to: shallow.time_to,
                add_date: shallow.add_date,
                id: shallow.id,
                content: shallow.content,
                date: shallow.date,
            })
        } else {
            let (category, created_by) = tokio::try_join!(category_fut, created_by_fut)?;
            Ok(Event {
                category,
                created_by,
                subject: None,
                time_from: shallow.time_from,
                time_to: shallow.time_to,
                add_date: shallow.add_date,
                id: shallow.id,
                content: shallow.content,
                date: shallow.date,
            })
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<EventId, ShallowEvent>,
    categories: CategoriesRepository,
    subjects: SubjectsRepository,
    users: UsersRepository,
}

impl EventsRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        let categories = CategoriesRepository::new(Arc::clone(&synergia_api));
        let users = UsersRepository::new(Arc::clone(&synergia_api));
        let subjects = SubjectsRepository::new(Arc::clone(&synergia_api));

        Self {
            synergia_api,
            categories,
            subjects,
            cache: AutoKeyedCache::new(),
            users,
        }
    }

    pub async fn list(&self) -> Result<Vec<Event>, Error> {
        if self.cache.size().await == 0 {
            self.cache
                .try_bulk_insert_with(async { self.synergia_api.events().fetch_list().await })
                .await
                .map_err(Error::EventListFetchError)?;
        }

        let shallow_events = self.cache.iter().map(|(_, v)| v).collect_vec();

        let event_factory = EventFactory::new(&self.categories, &self.users, &self.subjects);

        stream::iter(shallow_events)
            .map(async |s| event_factory.create_from_shallow(s).await)
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await
    }

    pub async fn fetch_from_calendar<'a>(
        &'a self,
        calendar: Calendar,
    ) -> Result<Vec<Event>, Error> {
        let event_factory = EventFactory::new(&self.categories, &self.users, &self.subjects);
        let shallow_events = stream::iter(calendar.events)
            .map(async |id| {
                self.cache
                    .try_get_with(&id, async {
                        self.synergia_api.events().fetch_event(id).await
                    })
                    .await
                    .map_err(Error::EventFetchError)
            })
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await?;

        stream::iter(shallow_events)
            .map(async |s| event_factory.create_from_shallow(s).await)
            .buffer_unordered(10)
            .try_collect::<Vec<_>>()
            .await
    }
}
