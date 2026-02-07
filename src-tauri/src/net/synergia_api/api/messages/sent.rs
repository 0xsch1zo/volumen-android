use base64::{prelude::BASE64_STANDARD, Engine};
use serde::Deserialize;

use crate::{
    net::synergia_api::api::messages::{Base64String, MessageId, MessageModelConversionError},
    repositories::messages::sent as models,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SentMessagePreview {
    message_id: MessageId,
    receiver_name: String,
    topic: String,
    #[serde(rename = "content")]
    fragment: Base64String,
    send_date: String,
    #[serde(rename = "isAnyFileAttached")]
    has_file_attachment: bool,
}

impl TryFrom<SentMessagePreview> for models::SentMessagePreview {
    type Error = MessageModelConversionError;

    fn try_from(value: SentMessagePreview) -> Result<Self, Self::Error> {
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
            receiver_name: value.receiver_name,
            send_date: value.send_date,
            topic: value.topic,
            has_file_attachment: value.has_file_attachment,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SentMessagePreviews {
    #[serde(rename = "data")]
    messages: Vec<SentMessagePreview>,
    total: usize,
}

impl TryFrom<SentMessagePreviews> for models::SentMessagePreviews {
    type Error = MessageModelConversionError;

    fn try_from(value: SentMessagePreviews) -> Result<Self, Self::Error> {
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
#[serde(rename_all = "camelCase")]
pub struct SentMessage {
    message_id: MessageId,
    sender_name: String,
    topic: String,
    #[serde(rename = "Message")] // at this point I'm not saying anything
    message: Base64String,
    send_date: String,
    read_date: Option<String>,
    no_reply: usize,
    archive: usize,
    attachments: Vec<super::AttachmentReference>,
    receivers: Vec<super::Receiver>,
}

impl TryFrom<SentMessage> for models::SentMessage {
    type Error = MessageModelConversionError;

    fn try_from(value: SentMessage) -> Result<Self, Self::Error> {
        let message_id = value
            .message_id
            .try_into()
            .map_err(MessageModelConversionError::MessageIdParsingError)?;
        let attachments: Vec<super::models::AttachmentReference> = value
            .attachments
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(MessageModelConversionError::AttachmentIdParsingError)?;
        let receivers: Vec<super::models::Receiver> = value
            .receivers
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        let message = BASE64_STANDARD
            .decode(value.message.0)
            .map_err(MessageModelConversionError::ContentDecodingError)?;
        let message =
            String::from_utf8(message).map_err(MessageModelConversionError::ContentUtf8Error)?;

        Ok(Self {
            message_id,
            message,
            attachments,
            receivers,
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
pub struct SentMessageResponse {
    data: SentMessage,
}

impl TryFrom<SentMessageResponse> for models::SentMessage {
    type Error = MessageModelConversionError;
    fn try_from(value: SentMessageResponse) -> Result<Self, Self::Error> {
        value.data.try_into()
    }
}
