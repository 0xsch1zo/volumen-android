use std::{num::ParseIntError, string::FromUtf8Error};

use base64::{prelude::BASE64_STANDARD, DecodeError, Engine};
use serde::Deserialize;
use serde_with::serde_as;
use thiserror::Error;

use crate::repositories::messages as models;

#[derive(Error, Debug)]
pub enum MessageModelConversionError {
    #[error("failed to parse message id as usize")]
    MessageIdParsingError(#[source] ParseIntError),
    #[error("failed to parse message id as usize")]
    AttachmentIdParsingError(#[source] ParseIntError),
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SentMessagePreview {
    message_id: MessageId,
    reciever_name: String,
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
            .map_err(MessageModelConversionError::MessageIdParsingError)?;
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

#[derive(Deserialize)]
#[serde_as]
#[serde(rename_all = "camelCase")]
pub struct RecievedMessage {
    message_id: MessageId,
    sender_name: String,
    topic: String,
    #[serde(rename = "Message")] // at this point I'm not saying anything
    message: Base64String,
    send_date: String,
    read_date: Option<String>,
    no_reply: usize,
    archive: usize,
    attachments: Vec<AttachmentReference>,
}

impl TryFrom<RecievedMessage> for models::RecievedMessage {
    type Error = MessageModelConversionError;

    fn try_from(value: RecievedMessage) -> Result<Self, Self::Error> {
        let message_id = value
            .message_id
            .try_into()
            .map_err(MessageModelConversionError::MessageIdParsingError)?;
        let attachments: Vec<models::AttachmentReference> = value
            .attachments
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(MessageModelConversionError::AttachmentIdParsingError)?;
        let message = BASE64_STANDARD
            .decode(value.message.0)
            .map_err(MessageModelConversionError::ContentDecodingError)?;
        let message =
            String::from_utf8(message).map_err(MessageModelConversionError::ContentUtf8Error)?;

        Ok(Self {
            message_id,
            message,
            attachments,
            sender_name: value.sender_name,
            send_date: value.send_date,
            topic: value.topic,
            read_date: value.read_date,
            no_reply: value.no_reply != 0, // thanks librus <3
            is_archived: value.archive != 0,
        })
    }
}

#[derive(Deserialize)]
pub struct RecievedMessageResponse {
    data: RecievedMessage,
}

impl TryFrom<RecievedMessageResponse> for models::RecievedMessage {
    type Error = MessageModelConversionError;
    fn try_from(value: RecievedMessageResponse) -> Result<Self, Self::Error> {
        value.data.try_into()
    }
}
