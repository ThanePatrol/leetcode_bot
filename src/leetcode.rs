use sqlx::Row;
use sqlx::sqlite::SqliteRow;
use crate::leetcode::Difficulty::{Easy, Hard, Medium};

#[derive(Debug, Clone)]
pub struct Leetcode {
    pub db_id: u32,
    pub name: String,
    pub url: String,
    pub number: u32,
    pub categories: Vec<String>,
    pub difficulty: Difficulty,
    pub have_solved: bool
}

impl Leetcode {
    pub fn new(name: String, url: String) -> Self {
        Self {
            db_id: 0,
            name,
            url,
            number: 0,
            categories: vec![],
            difficulty: Difficulty::Easy,
            have_solved: false,
        }
    }

    // creates a new question instance from a row fetched from DB
    pub fn new_from_row(row: SqliteRow) -> Self {
        Self {
            db_id: row.get::<i32, _>(0) as u32,
            name: row.get::<String, _>(2),
            url: row.get::<String, _>(3),
            number: row.get::<i32, _>(1) as u32,
            categories: serde_json::de::from_slice(&*row.get::<Vec<u8>, _>(5)).unwrap(),
            difficulty: Difficulty::new(row.get::<String, _>(4)),
            have_solved: row.get::<bool, _>(6),
        }
    }

    pub fn serialize_categories(&self) -> String {
        serde_json::ser::to_string(&self.categories)
            .expect("Error serializing category")
    }


}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard
}

impl Difficulty {
    /// used for creating a new enum or deserializing from db
    pub fn new(input: String) -> Self {
        match input.as_str() {
            "Easy" => Easy,
            "Medium" => Medium,
            _ => Hard,
        }
    }

    pub fn serialize_to_str(&self) -> String {
        let str = match self {
            Easy => "Easy",
            Medium => "Medium",
            Hard => "Hard"
        };
        serde_json::ser::to_string(str)
            .expect("Error serializing enum to str")
    }
}

#[cfg(test)]
mod tests {
    use crate::db_api;
    use super::*;

    #[tokio::test]
    async fn test_read_question_from_db() -> Result<(), sqlx::Error>{
        let env_file = dotenvy::dotenv().expect("Could not read .env file");
        let db_url = std::env::var("DATABASE_URL").expect("Error reading db url from .env");
        let pool = db_api::init_db(&db_url).await?;

        let two_sum = db_api::get_question_from_url(
            &"https://leetcode.com/problems/two-sum/".to_string(), &pool
        ).await?;
        println!("{:?}", two_sum);

        assert_eq!(two_sum.name, "Two Sum".to_string());
        assert_eq!(two_sum.difficulty, Difficulty::Easy);
        assert_eq!(two_sum.number, 1);
        assert_eq!(two_sum.categories, vec!["Array".to_string(), "Hash Table".to_string()]);
        Ok(())
    }
}