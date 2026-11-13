use std::num::ParseIntError;

use base64::{prelude::BASE64_STANDARD, DecodeError, Engine};
use serde::Deserialize;
use thiserror::Error;

use crate::repositories::messages as models;

pub mod received;
pub mod sent;

#[derive(Error, Debug)]
pub enum MessageModelConversionError {
    #[error("failed to parse message id as usize")]
    MessageIdParsingError(#[source] ParseIntError),
    #[error("failed to parse message id as usize")]
    AttachmentIdParsingError(#[source] ParseIntError),
    #[error("failed to parse message id as usize")]
    ReceiverIdParsingError(#[source] ParseIntError),
    #[error("failed to decode message contents from base64")]
    ContentDecodingError(#[source] DecodeError),
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct MessageId(String);

impl TryFrom<MessageId> for models::MessageId {
    type Error = ParseIntError;

    fn try_from(value: MessageId) -> Result<Self, Self::Error> {
        Ok(Self::new(value.0.parse::<usize>()?))
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct Base64String(String);

impl Base64String {
    pub fn to_string_lossy(&self) -> Result<String, MessageModelConversionError> {
        Ok(String::from_utf8_lossy(
            &BASE64_STANDARD
                .decode(&self.0)
                .map_err(MessageModelConversionError::ContentDecodingError)?,
        )
        .to_string())
    }
}

#[derive(Deserialize)]
#[serde(transparent)]
struct AttachmentId(String);

impl TryFrom<AttachmentId> for models::AttachmentId {
    type Error = ParseIntError;

    fn try_from(value: AttachmentId) -> Result<Self, Self::Error> {
        Ok(Self::new(value.0.parse()?))
    }
}

#[derive(Deserialize)]
struct AttachmentReference {
    id: AttachmentId,
    filename: String,
}

impl TryFrom<AttachmentReference> for models::AttachmentReference {
    type Error = ParseIntError;

    fn try_from(value: AttachmentReference) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.try_into()?,
            filename: value.filename,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
struct ReceiverId(String);

impl TryFrom<ReceiverId> for models::ReceiverId {
    type Error = ParseIntError;

    fn try_from(value: ReceiverId) -> Result<Self, Self::Error> {
        Ok(Self::new(value.0.parse()?))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Receiver {
    receiver_id: ReceiverId,
    name: String,
    #[serde(rename = "readed")] // because proper english is too much for librus dev's
    read_date: String,
}

impl TryFrom<Receiver> for models::Receiver {
    type Error = MessageModelConversionError;

    fn try_from(value: Receiver) -> Result<Self, Self::Error> {
        Ok(Self {
            receiver_id: value
                .receiver_id
                .try_into()
                .map_err(MessageModelConversionError::ReceiverIdParsingError)?,
            name: value.name,
            read_date: value.read_date,
        })
    }
}
