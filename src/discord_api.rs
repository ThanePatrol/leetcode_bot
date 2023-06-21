use std::collections::VecDeque;
use std::error::Error;
use std::rc::Rc;
use discord::Discord;
use discord::model::ChannelId;
use sqlx::{Pool, Sqlite};
use crate::db_api;
use crate::leetcode::Leetcode;

/// A queue like structure that can be appended to with discord commands
/// the front of the queue will be the next question posted
/// operations will be to add to front of queue or to end.
/// Should not be interacted with directly via the user, only via the DiscordAPI struct.
/// This is so the user only has a limited API to work with
/// Queue acts as a singleton, there is no global state for it however,
/// it should be instantiated in main then passed around when necessary
pub struct QuestionQueue {
    pub queue: VecDeque<Leetcode>,
    pool: Rc<Pool<Sqlite>>,
}

impl QuestionQueue {
    pub fn new(pool: Rc<Pool<Sqlite>>) -> Self {
        QuestionQueue {
            queue: Default::default(),
            pool,
        }

    }

    /// Provides the next question, removes it from the queue
    /// if there is nothing in the queue it selects a random question.
    /// In both cases the question is marked as completed
    pub async fn get_next_question(&mut self) -> Result<Leetcode, sqlx::Error>{
        let question = match self.queue.pop_front() {
            Some(q) => q,
            None => db_api::get_random_question_from_db(self.pool.as_ref()).await?,
        };

        db_api::mark_question_as_done(question.db_id as i32, self.pool.as_ref()).await?;

        Ok(question)
    }

    pub async fn push_to_back(&mut self, problem_url: &String) -> Result<(), sqlx::Error> {
        let question = db_api::get_question_from_url(problem_url, self.pool.as_ref()).await?;
        self.queue.push_back(question);
        Ok(())
    }

    //todo - serialize struct back again
    pub fn save_queue_state() {

    }

    pub fn get_current_questions_in_queue(&self) -> Vec<Leetcode> {
        self.queue.iter().cloned().collect()
    }

}

/// Provides a thin wrapper around the Discord crate.
/// Allows users to issue discord commands to a specific channel,
/// these channels are defined at creation time of the struct
pub struct DiscordAPI {
    client: Rc<Discord>,
    command_channel_id: u64,
    question_channel_id: u64,
}

impl DiscordAPI {
    pub fn new(client: Rc<Discord>, command_channel: u64, question_channel: u64) -> Self {
        Self {
            client,
            command_channel_id: command_channel,
            question_channel_id: question_channel,
        }
    }

    pub async fn ping_with_daily(&self, role_id: u64, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        let question = question_queue.get_next_question().await?;

        let msg = format!("<@&{}> The daily question is {}", role_id, question.url.clone());
        self.client.send_message(
            ChannelId(self.question_channel_id),
            &*msg,
            "",
            false,
        )?;

        Ok(())
    }

    /// Assumes the problem_url is in the data base, will fail if it is not in
    /// //todo - give a soft error message instead
    pub async fn add_question_to_queue(&self, problem_url: &String, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        question_queue.push_to_back(problem_url).await?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_pop() {
        let env_file = dotenvy::dotenv().expect("Could not read .env file");
        let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
        let pool = Rc::new(db_api::init_db(&db_url).await.unwrap());

        let two_sum = "https://leetcode.com/problems/two-sum/".to_string();

        let mut queue = QuestionQueue::new(pool.clone());
        queue.push_to_back(&two_sum).await.unwrap();

        assert_eq!(queue.queue.len(), 1);
        assert_eq!(queue.queue.front().unwrap().have_solved, false);

        let question = queue.get_next_question().await.unwrap();
        assert_eq!(question.name, "Two Sum".to_string());

        let question = db_api::get_question_from_url(&two_sum, pool.as_ref()).await.unwrap();
        assert_eq!(question.have_solved, true);

        db_api::mark_question_as_not_completed(question.db_id as i32, pool.as_ref()).await.unwrap();


    }
}

