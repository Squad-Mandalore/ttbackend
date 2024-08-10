use crate::models;

pub async fn get_timers(
    employee_id: i32,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> sqlx::Result<Vec<models::Worktime>> {
    sqlx::query_as!(
        models::Worktime,
        "SELECT * FROM worktime WHERE employee_id = $1",
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn start_timer(
    employee_id: i32,
    worktype: models::WorktimeType,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> sqlx::Result<models::Worktime> {
    sqlx::query_as!(models::Worktime, "INSERT INTO worktime(employee_id, start_time, type) values($1, $2, $3) RETURNING employee_id", employee_id, time_utils::create_timestamp(), worktype).fetch_one(pool).await
}
