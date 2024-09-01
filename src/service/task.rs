use crate::models;

pub(crate) async fn get_task_by_id(
    task_id: i32,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Option<models::Task>> {
    sqlx::query_as!(
        models::Task,
        "SELECT task_id, task_description FROM task WHERE task_id = $1",
        task_id
    )
    .fetch_optional(pool)
    .await
}

pub(crate) async fn get_tasks(pool: &sqlx::PgPool) -> sqlx::Result<Vec<models::Task>> {
    sqlx::query_as!(models::Task, "SELECT task_id, task_description FROM task")
        .fetch_all(pool)
        .await
}

pub(crate) async fn create_task(
    task_description: String,
    pool: &sqlx::PgPool,
) -> sqlx::Result<models::Task> {
    sqlx::query_as!(
        models::Task,
        "INSERT INTO task (task_description) VALUES($1) RETURNING task_id, task_description",
        task_description
    )
    .fetch_one(pool)
    .await
}

pub(crate) async fn update_task(
    task_id: i32,
    task_description: String,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Option<models::Task>> {
    sqlx::query_as!(models::Task, "UPDATE task SET task_description = $2 WHERE task_id = $1 RETURNING task_id, task_description", task_id, task_description).fetch_optional(pool).await
}

pub(crate) async fn delete_task(
    task_id: i32,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Option<models::Task>> {
    sqlx::query_as!(
        models::Task,
        "DELETE FROM task WHERE task_id = $1 RETURNING task_id, task_description",
        task_id
    )
    .fetch_optional(pool)
    .await
}
