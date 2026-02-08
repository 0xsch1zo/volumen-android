use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{AuthenticatedState, AuthenticatedSynergiaApiError},
        SynergiaApi,
    },
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to fetch comment with id: {1:?}")]
    CommentFetchFailed(#[source] CacheComputeError, CommentId),
}

#[derive(Serialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct CommentId(usize);

impl CommentId {
    pub fn new(_0: usize) -> Self {
        Self(_0)
    }

    pub fn inner(&self) -> usize {
        self.0
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Comment {
    pub id: CommentId,
    pub text: String,
}

impl Keyable<CommentId> for Comment {
    fn key(&self) -> CommentId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct CommentsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CommentId, Comment>,
}

impl CommentsRepository {
    pub fn new(synergia_api: Arc<SynergiaApi<AuthenticatedState>>) -> Self {
        Self {
            synergia_api,
            cache: AutoKeyedCache::new(),
        }
    }

    pub async fn comment(&self, id: CommentId) -> Result<Comment, Error> {
        if let Some(comment) = self.cache.get(&id).await {
            return Ok(comment.clone());
        }

        let comment = self
            .cache
            .try_get_with(&id, async {
                let comment = self.synergia_api.grades().fetch_comment(id).await?;
                Ok::<_, AuthenticatedSynergiaApiError>(comment)
            })
            .await
            .map_err(|e| Error::CommentFetchFailed(e, id))?;
        Ok(comment)
    }
}
