use itertools::{EitherOrBoth, Itertools};
use log::{debug, warn};
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use crate::{
    common::TakeExactlyExt,
    net::librus_api::synergia_api::{Message, Messages},
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("message parsing failed")]
    MessageParseFailed(#[from] MessageParseError),
}

#[derive(Error, Debug)]
pub enum MessageParseError {
    #[error("the message table doesn't contain enough columns")]
    NotEnoughColumns,
    #[error("text not found in the message column: {0:?}")]
    TextNotFound(MessageColumns),
    #[error("full message link not found")]
    FullMessageLinkNotFound,
}

#[derive(Clone, Debug)]
pub enum MessageColumns {
    Sender = 0,
    Title = 1,
    Date = 2,
}

fn parse_message<'a>(value: ElementRef<'a>) -> Result<Message, MessageParseError> {
    const CORRECT_MESSAGE_COLUMN_AMOUNT: usize = 6;
    const USELESS_COLUMNS_BEFORE_COUNT: usize = 2;
    const USEFUL_COLUMNS_COUNT: usize = 3;

    let row_selector = Selector::parse("td").unwrap();
    let column_count = value.select(&row_selector).count();
    match column_count {
        0..CORRECT_MESSAGE_COLUMN_AMOUNT => return Err(MessageParseError::NotEnoughColumns),
        CORRECT_MESSAGE_COLUMN_AMOUNT => {}
        _ => warn!("there is more message columns in the document than there should be"),
    };

    let message_columns = value
        .select(&row_selector)
        .skip(USELESS_COLUMNS_BEFORE_COUNT)
        .take(USEFUL_COLUMNS_COUNT)
        .collect_vec();

    let [sender, title, date] = [
        MessageColumns::Sender,
        MessageColumns::Title,
        MessageColumns::Date,
    ]
    .map(|c| (c.clone() as usize, c))
    .map(|(i, c)| {
        message_columns[i]
            .text()
            .filter(|s| !s.trim().is_empty())
            .take_exactly::<Vec<&str>>(1)
            .inspect_surplus(|_| warn!("{c:?} column text surplus"))
            .enough_or(MessageParseError::TextNotFound(c))
    });

    let endpoint_hyperlink_selector = Selector::parse("a[href]").unwrap();
    let full_message_endpoint = message_columns[MessageColumns::Title as usize]
        .select(&endpoint_hyperlink_selector)
        .take_exactly::<Vec<ElementRef>>(1)
        .inspect_surplus(|_| warn!("full message link surplus"))
        .enough_or(MessageParseError::FullMessageLinkNotFound)?
        .into_iter()
        .exactly_one()
        .unwrap()
        .attr("href")
        .ok_or(MessageParseError::FullMessageLinkNotFound)?
        .to_owned();

    Ok(Message {
        full_message_endpoint,
        title: title?[0].to_owned(),
        sender: sender?[0].to_owned(),
        date: date?[0].to_owned(), // get rid of this use lifetimes
    })
}

pub fn scrape_messages(messages_html: &str) -> Result<Messages, Error> {
    debug!("scraping messages");
    let container_selector = Selector::parse("table.decorated.stretch").unwrap();
    let even_message_selector = Selector::parse("tr.line0").unwrap();
    let odd_message_selector = Selector::parse("tr.line1").unwrap();

    let document = Html::parse_document(messages_html);
    if document.select(&container_selector).count() != 1 {
        warn!("more than one message container detected");
    }

    let container_fragment = document.select(&container_selector).next().unwrap();
    let messages: Vec<Message> = Vec::new();

    debug!("successfully scraped messages");

    Ok(Messages {
        messages: container_fragment
            .select(&even_message_selector)
            .zip_longest(container_fragment.select(&odd_message_selector))
            .try_fold(messages, |mut acc, message_pair| {
                match message_pair {
                    EitherOrBoth::Left(m) => acc.push(parse_message(m)?),
                    EitherOrBoth::Right(m) => acc.push(parse_message(m)?),
                    EitherOrBoth::Both(m1, m2) => {
                        acc.push(parse_message(m1)?);
                        acc.push(parse_message(m2)?);
                    }
                };
                Ok::<Vec<Message>, MessageParseError>(acc)
            })?,
    })
}
