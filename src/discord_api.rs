use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use discord::Discord;
use discord::model::{ChannelId, Message};
use sqlx::{Pool, Sqlite};
use crate::db_api;
use crate::discord_api::CommandType::{AddQuestion, PostQuestion, ViewQuestions};
use crate::leetcode::{Difficulty, Leetcode};

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
    pub async fn get_next_question(&mut self) -> Result<Leetcode, sqlx::Error> {
        let question = match self.queue.pop_front() {
            Some(q) => q,
            None => db_api::get_random_question_from_db(self.pool.as_ref()).await?,
        };

        db_api::mark_question_as_done(question.db_id as i32, self.pool.as_ref()).await?;

        Ok(question)
    }

    pub async fn push_url_to_back(&mut self, problem_url: &String) -> Result<(), sqlx::Error> {
        let question = db_api::get_question_from_url(problem_url, self.pool.as_ref()).await?;
        self.queue.push_back(question);
        Ok(())
    }

    pub async fn push_question_num_to_back(&mut self, problem_num: i32) -> Result<(), sqlx::Error> {
        let question = db_api::get_question_from_number(problem_num, self.pool.as_ref()).await?;
        self.queue.push_back(question);
        Ok(())
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
    pub command_channel_id: u64,
    pub question_channel_id: u64,
    pub role_id_easy: u64,
    pub role_id_med: u64,
    pub role_id_hard: u64,
    pub bot_id: u64,
    announcement_text: String,
}

impl DiscordAPI {
    pub fn new(
        client: Rc<Discord>,
        command_channel: u64,
        question_channel: u64,
        role_id_easy: u64,
        role_id_med: u64,
        role_id_hard: u64,
        bot_id: u64,
        announcement_text: String,
    ) -> Self {
        Self {
            client,
            command_channel_id: command_channel,
            question_channel_id: question_channel,
            role_id_easy,
            role_id_med,
            role_id_hard,
            bot_id,
            announcement_text,
        }
    }

    pub async fn ping_with_daily(&self, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        let question = question_queue.get_next_question().await?;

        let thread_name = DiscordAPI::build_thread_name(&question);

        let role_id = match question.difficulty {
            Difficulty::Easy => self.role_id_easy,
            Difficulty::Medium => self.role_id_med,
            Difficulty::Hard => self.role_id_hard
        };

        let msg = format!("<@&{}> {} {}", role_id, self.announcement_text, question.url.clone());
        let message = self.client.send_message(
            ChannelId(self.question_channel_id),
            &*msg,
            "",
            false,
        )?;

        Self::create_new_thread_with_message(self, message, &thread_name).await?;

        Ok(())
    }

    fn build_thread_name(question: &Leetcode) -> String {
        let mut thread_name = String::new();
        match question.difficulty {
            Difficulty::Easy => thread_name.push_str("\u{1F7E2} "),
            Difficulty::Medium => thread_name.push_str("\u{1F7E1} "),
            Difficulty::Hard => thread_name.push_str("\u{1F534} "),
        }
        thread_name.push_str(&*question.number.to_string());
        thread_name.push_str(". ");
        thread_name.push_str(&*question.name);

        thread_name
    }

    pub async fn get_all_questions_in_queue(&self, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        let questions = question_queue.get_current_questions_in_queue();
        self.client.as_ref()
            .send_message(
                ChannelId(self.command_channel_id),
                &*format!("The questions are: {:?}", questions),
                "",
                false)?;
        Ok(())
    }

    /// Assumes the question is in the data base, will fail if it is not present.
    /// Assumes question is in format push..`url` OR push..`num`
    /// where `url` is in the format https://leetcode.com/problems/two-sum/description/
    /// and where `num` is the number of the question, eg 1 for Two Sum
    pub async fn add_question_to_queue(&self, user_input: &String, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        if Self::is_numeric(user_input.as_ref()) {
            self.add_question_as_number_to_queue(user_input, question_queue).await?
        } else {
            self.add_question_as_url_to_queue(user_input, question_queue).await?
        }
        Ok(())
    }

    async fn add_question_as_url_to_queue(&self, url: &String, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        if let Some(url) = Self::split_problem_on_dots(url.as_ref()) {
            match question_queue.push_url_to_back(&url.to_string()).await {
                Ok(_) => {}
                Err(_) => {
                    return Err(Box::new(UserError("Unrecognised problem, make sure problem url is \
                    in the format https://leetcode.com/problems/two-sum/ \
                    and the problem is in the database. \n Note that the url \
                    does not have /description after it".to_string())));
                }
            }
        } else {
            return Err(Box::new(UserError("Ensure command is in format: push..`url`".to_string())));
        }
        Ok(())
    }

    /// Assumes number is strictly numeric
    async fn add_question_as_number_to_queue(&self, number: &String, question_queue: &mut QuestionQueue) -> Result<(), Box<dyn Error>> {
        let number = Self::split_problem_on_dots(number)
            .unwrap_or("-1")
            .parse::<i32>()
            .unwrap_or(-1);
        match question_queue.push_question_num_to_back(number).await {
            Ok(_) => {}
            Err(_) => {
                return Err(Box::new(UserError("Unrecognised problem, make sure problem number is \
                    correct and the problem is in the database".to_string())));
            }
        }

        Ok(())
    }

    /// Creates a new thread for the question from the message
    async fn create_new_thread_with_message(
        &self,
        message: Message,
        thread_name: &String,
    )
        -> Result<(), Box<dyn Error>> {
        let _ = self.client.as_ref()
            .create_thread(
                ChannelId(self.question_channel_id),
                message.id,
                |ch| ch.name(thread_name.as_str()))?;
        Ok(())
    }


    /// Trims and splits a users discord command
    fn split_problem_on_dots(user_input: &str) -> Option<&str> {
        let trimmed = user_input.trim();
        let split = trimmed.split("..").last()?;
        Some(split)
    }

    /// Check if part after the dots is numeric
    /// unwrap_or("a") as we want to handle errors in the url parsing section
    /// returning false here will force the url handler to deal with incorrect errors
    /// rather than duplicating work
    fn is_numeric(input: &str) -> bool {
        input.trim()
            .split("..")
            .last()
            .unwrap_or("a")
            .chars()
            .all(|c| c.is_numeric())
    }

    /// Reads the first part of the command, returning an enum to dictate what type of command
    /// was entered. The two supported commands are 'push' and 'pop'
    /// Push adds the question to the queue, pop will get the thing at the front of the queue
    pub fn parse_command(user_input: &String) -> Result<CommandType, UserError> {
        let trimmed = user_input.trim();

        if let Some(split) = trimmed.split("..").next() {
            match split {
                "push" => Ok(AddQuestion),
                "pop" => Ok(PostQuestion),
                "view" => Ok(ViewQuestions),
                _ => Err(UserError("Ensure command is in format: action..".to_string())),
            }
        } else {
            Err(UserError("Ensure command is in format: action..".to_string()))
        }
    }

    pub fn send_error_message(&self, error: Box<dyn Error>) {
        self.client.send_message(
            ChannelId(self.command_channel_id),
            &*error.to_string(),
            "",
            false,
        ).expect("Couldn't send error message...");
    }

    pub fn send_confirmation_message(&self, text: &str) {
        self.client.send_message(
            ChannelId(self.command_channel_id),
            text,
            "",
            false,
        ).expect("Couldn't send confirmation message");
    }
}

pub enum CommandType {
    AddQuestion,
    PostQuestion,
    ViewQuestions,
}

#[derive(Debug)]
pub struct UserError(String);


impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid user input, potentially skill issue: {}", self.0)
    }
}

impl Error for UserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_pop() {
        dotenvy::dotenv().expect("Could not read .env file");
        let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
        let pool = Rc::new(db_api::init_db(&db_url).await.unwrap());

        let two_sum = "https://leetcode.com/problems/two-sum/".to_string();

        let mut queue = QuestionQueue::new(pool.clone());
        queue.push_url_to_back(&two_sum).await.unwrap();

        assert_eq!(queue.queue.len(), 1);
        assert_eq!(queue.queue.front().unwrap().have_solved, false);

        let question = queue.get_next_question().await.unwrap();
        assert_eq!(question.name, "Two Sum".to_string());

        let question = db_api::get_question_from_url(&two_sum, pool.as_ref()).await.unwrap();
        assert_eq!(question.have_solved, true);

        db_api::mark_question_as_not_completed(question.db_id as i32, pool.as_ref()).await.unwrap();
    }
}

