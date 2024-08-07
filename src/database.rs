use std::time::Duration;

use sqlx::{Pool, Postgres};

pub async fn set_up_connection_pool() -> Pool<Postgres> {
    let database_url = dotenvy::var("DATABASE_URL").expect("there is no .env file");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("can't connect to database")
}


//     sqlx::migrate!().run(&db).await?;

//     sqlx_example_postgres_axum_social::http::serve(db).await
