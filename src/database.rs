use std::time::Duration;

pub async fn set_up_database() -> sqlx::PgPool {
    let database_url =
        dotenvy::var("DATABASE_URL").expect("there is no .env file or no DATABASE_URL present");
    let database_pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("can't connect to database");

    sqlx::migrate!("./migrations")
        .run(&database_pool)
        .await
        .expect("cannot run migrations");

    database_pool
}
