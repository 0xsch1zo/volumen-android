use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cache::{AutoKeyedCache, CacheComputeError, Keyable},
    net::{
        synergia_api::{self, AuthenticatedState},
        SynergiaApi,
    },
    repositories::entities::Reference,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("comment with id: {0:?}, not found")]
    CommentNotFound(CommentId),
    #[error("failed to fetch comment with id: {1:?}")]
    CommentFetchFailed(#[source] CacheComputeError, CommentId),
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, Debug)]
#[serde(from = "Reference")]
pub struct CommentId(usize);

impl CommentId {
    pub fn inner(&self) -> usize {
        self.0
    }
}

impl From<Reference> for CommentId {
    fn from(value: Reference) -> Self {
        Self(match value {
            Reference::Linked { id, .. } => id,
            Reference::Standalone(id) => id,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Comment {
    pub id: CommentId,
    pub text: String,
}

impl Keyable<CommentId> for Comment {
    fn key(&self) -> CommentId {
        self.id
    }
}

pub struct CommentsRepository {
    synergia_api: Arc<SynergiaApi<AuthenticatedState>>,
    cache: AutoKeyedCache<CommentId, Comment>,
}

impl CommentsRepository {
    pub async fn comment(&self, id: CommentId) -> Result<Comment, Error> {
        if let Some(comment) = self.cache.get(&id).await {
            return Ok(comment.clone());
        }

        let comment = self
            .cache
            .try_get_with(&id, async {
                let comment = self.synergia_api.comment(id).await?;
                Ok::<_, synergia_api::Error>(comment)
            })
            .await
            .map_err(|e| Error::CommentFetchFailed(e, id))?;
        Ok(comment)
    }
}
