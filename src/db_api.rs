use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;
use crate::leetcode::Leetcode;

pub async fn init_db(db_url: &String) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

pub async fn add_leetcode_entries_to_db(
    questions: Vec<Leetcode>,
    pool: &Pool<Sqlite>,
)
    -> Result<(), sqlx::Error> {

    for question in questions {
        sqlx::query (
            "INSERT OR IGNORE INTO leetcode (problem_name, problem_link, have_done) \
            VALUES (?, ?, ?);")
            .bind(question.name)
            .bind(question.url)
            .bind(question.have_solved)
            .execute(pool)
            .await?;
    }
    Ok(())
}
