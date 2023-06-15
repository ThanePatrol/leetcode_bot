use sqlx::{Pool, Sqlite};
use sqlx::sqlite::SqlitePoolOptions;

pub async fn init_db(db_url: &String) -> Result<Pool<Sqlite>, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}


// pub async fn insert_leetcode_into_db() -> Result<(), sqlx::Error> {
//
// }