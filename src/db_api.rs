use sqlx::{Pool, Row, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;
use crate::leetcode::Leetcode;

pub async fn init_db(db_url: &String) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

//todo - where i got up to
// adding the updated leetcode entries into the db
// need to serialize the question categories
// also should change table names once done
pub async fn add_leetcode_entries_to_db(
    questions: Vec<Leetcode>,
    pool: &Pool<Sqlite>,
)
    -> Result<(), sqlx::Error> {

    for question in questions {
        sqlx::query (
            "INSERT OR IGNORE INTO leetcode2 (problem_num, problem_name, problem_link, difficulty,\
             problem_categories, have_done) \
            VALUES (?, ?, ?, ?, ?, ?);")
            .bind(question.name)
            .bind(question.url)
            .bind(question.have_solved)
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// id is auto-incrementing sql id, not leetcode number
/// assumes table layout
/// //todo - change to fit new table
// pub async fn have_done_question(id: i32, pool: &Pool<Sqlite>) -> Result<bool, sqlx::Error> {
//     let row = sqlx::query (
//         "SELECT * FROM leetcode WHERE id == ?;")
//         .bind(id)
//         .fetch(pool)
//         .await?;
//     Ok(row.get::<bool, _>(4))
// }

pub async fn get_all_questions(pool: &Pool<Sqlite>) -> Result<Vec<Leetcode>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM leetcode;"
    )
        .fetch_all(pool)
        .await?;

    let mut questions = Vec::new();
    for row in rows {
        let question = Leetcode::new(row.get::<String, _>(1), row.get::<String, _>(2));
        questions.push(question);
    }
    Ok(questions)
}

