use rand::Rng;
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

#[allow(dead_code)]
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

/// Assumes the question parameter is a url in the form https://leetcode.com/problems/two-sum/
pub async fn get_question_from_url(question: &String, pool: &Pool<Sqlite>) -> Result<Leetcode, sqlx::Error> {
    let row = sqlx::query("select * from leetcode where problem_link == ?;")
        .bind(question)
        .fetch_one(pool)
        .await?;

    Ok(Leetcode::new_from_row(row))
}

/// Assumes question is strictly numeric and question is in the database
pub async fn get_question_from_number(question_number: i32, pool: &Pool<Sqlite>) -> Result<Leetcode, sqlx::Error> {
    let row = sqlx::query("select * from leetcode where problem_num == ?;")
        .bind(question_number)
        .fetch_one(pool)
        .await?;
    Ok(Leetcode::new_from_row(row))
}


/// Retries a random question from the database that is not marked as completed
pub async fn get_random_question_from_db(pool: &Pool<Sqlite>) -> Result<Leetcode, sqlx::Error> {
    let mut rows = sqlx::query(
        "select * from leetcode where have_done == false;"
    )
        .fetch_all(pool)
        .await?;

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..rows.len());
    let row = rows.swap_remove(idx);
    let random_question = Leetcode::new_from_row(row);

    Ok(random_question)
}

/// uses database id to mark question as completed
pub async fn mark_question_as_done(id: i32, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE leetcode SET have_done = true WHERE id == ?;"
    )
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// uses database id to mark question as not completed. Mainly used for testing.
#[allow(dead_code)]
pub async fn mark_question_as_not_completed(id: i32, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE leetcode SET have_done = false WHERE id == ?;"
    )
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Used for updating db after a scrape of questions
/// Make public for general use
#[allow(dead_code)]
pub async fn add_leetcode_entries_to_db(
    questions: Vec<Leetcode>,
    pool: &Pool<Sqlite>,
)
    -> Result<(), sqlx::Error> {
    for question in questions {
        let difficulty = question.difficulty.serialize_to_str();
        let problem_categories = question.serialize_categories();

        // need to check if exists, don't update question if it does
        let does_exist = sqlx::query(
            "SELECT COUNT(1) FROM leetcode WHERE problem_num = ?"
        )
            .bind(question.number)
            .fetch_one(pool)
            .await?;
        // question already exists
        if does_exist.get::<i32, _>(0) == 1 {
            continue
        }

        sqlx::query(
            "INSERT OR IGNORE INTO leetcode (problem_num, problem_name, problem_link, difficulty,\
             problem_categories, have_done) \
            VALUES (?, ?, ?, ?, ?, ?);")
            .bind(question.number)
            .bind(question.name)
            .bind(question.url)
            .bind(difficulty)
            .bind(problem_categories)
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn add_grind75_entries_to_db(
    questions: Vec<Leetcode>,
    pool: &Pool<Sqlite>,
)
    -> Result<(), sqlx::Error> {
    for question in questions {
        sqlx::query(
            "INSERT OR IGNORE INTO grind (problem_name, problem_link) \
            VALUES (?, ?);")
            .bind(question.name)
            .bind(question.url)
            .execute(pool)
            .await?;
    }
    Ok(())
}

