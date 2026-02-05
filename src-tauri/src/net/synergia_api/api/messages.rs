use std::{num::ParseIntError, string::FromUtf8Error};

use base64::{prelude::BASE64_STANDARD, DecodeError, Engine};
use serde::Deserialize;
use thiserror::Error;

use crate::repositories::messages as models;

#[derive(Error, Debug)]
pub enum MessageModelConversionError {
    #[error("failed to parse message id as usize")]
    IdParsingError(#[source] ParseIntError),
    #[error("failed to decode message contents from base64")]
    ContentDecodingError(#[source] DecodeError),
    #[error("failed to convert messages contents to string due to invalid utf-8")]
    ContentUtf8Error(#[source] FromUtf8Error),
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

// skipped fields: tags, category, otherNodeUuid, otherNodeAccountId, sender_last_name,
// sender_first_name
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecievedMessagePreview {
    message_id: MessageId,
    sender_name: String,
    topic: String,
    #[serde(rename = "content")]
    fragment: Base64String,
    send_date: String,
    read_date: Option<String>,
    #[serde(rename = "isAnyFileAttached")]
    has_file_attachment: bool,
}

impl TryFrom<RecievedMessagePreview> for models::RecievedMessagePreview {
    type Error = MessageModelConversionError;

    fn try_from(value: RecievedMessagePreview) -> Result<Self, Self::Error> {
        let message_id = value
            .message_id
            .try_into()
            .map_err(MessageModelConversionError::IdParsingError)?;
        let fragment = BASE64_STANDARD
            .decode(value.fragment.0)
            .map_err(MessageModelConversionError::ContentDecodingError)?;
        let fragment =
            String::from_utf8(fragment).map_err(MessageModelConversionError::ContentUtf8Error)?;

        Ok(Self {
            message_id,
            fragment,
            sender_name: value.sender_name,
            send_date: value.send_date,
            topic: value.topic,
            read_date: value.read_date,
            has_file_attachment: value.has_file_attachment,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecievedMessagePreviews {
    #[serde(rename = "data")]
    messages: Vec<RecievedMessagePreview>,
    total: usize,
}

impl TryFrom<RecievedMessagePreviews> for models::RecievedMessagePreviews {
    type Error = MessageModelConversionError;

    fn try_from(value: RecievedMessagePreviews) -> Result<Self, Self::Error> {
        Ok(Self {
            messages: value
                .messages
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            total: value.total,
        })
    }
}
