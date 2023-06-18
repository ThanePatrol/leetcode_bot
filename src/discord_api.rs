use std::collections::VecDeque;
use discord::Discord;
use discord::model::ChannelId;
use crate::leetcode::Leetcode;

/// A queue like structure that can be appended to with discord commands
/// the front of the queue will be the next question asked
/// operations will be to add to front of queue or to end
/// Queue acts as a singleton, there is no global state for it however,
/// it should be instantiated in main then passed around when necessary
pub struct QuestionQueue {
    pub queue: VecDeque<Leetcode>
}

impl QuestionQueue {
    /// returns the deserialized queue
    /// assumes .env environment
    pub fn new() -> Self {
        let path = std::env::var("QUESTION_QUEUE")
            .expect("Error queue from .env");
        QuestionQueue {
            queue: Default::default(),
        }

    }

    //todo - database update
    pub fn mark_question_as_done() {

    }

    //todo - take discord command and push a question to the back of a queue
    pub fn push_to_back() {

    }

    //todo - serialize struct back again
    pub fn save_queue_state() {

    }

}

pub fn ping_with_daily(channel_id: u64, role_id: u64, link: &str, client: &Discord) -> Result<(), Box<dyn std::error::Error>> {
    let msg = format!("<@&{}> The daily question is {}", role_id, link);
    client.send_message(
        ChannelId(channel_id),
        &*msg,
        "",
        false,
    )?;

    Ok(())
}