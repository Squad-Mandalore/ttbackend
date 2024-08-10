use async_graphql::{Error, Result};

use crate::{models, utils::internal_error};

pub async fn get_timers(
    employee_id: i32,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> sqlx::Result<Vec<models::Worktime>> {
    sqlx::query_as!(
        models::Worktime,
        "SELECT employee_id FROM worktime WHERE employee_id = $1",
        employee_id
    )
    .fetch_all(pool)
    .await
}
