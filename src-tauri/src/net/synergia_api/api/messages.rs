use serde::Deserialize;

// skipped fields: tags, category, otherNodeUuid, otherNodeAccountId,
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    message_id: String,
    sender_first_name: String,
    sender_last_name: String,
    sender_name: String,
    topic: String,
    content: String,
    send_date: String,
    read_date: Option<String>,
    is_any_file_attached: bool,
}

#[derive(Deserialize)]
pub struct Messages {
    #[serde(rename = "data")]
    messages: Vec<Message>,
    total: usize,
}
