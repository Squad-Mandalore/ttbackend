use crate::models;

pub async fn get_timers(
    employee_id: i32,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Vec<models::Worktime>> {
    sqlx::query_as!(
        models::Worktime,
        r#"SELECT worktime_id, employee_id, task_id, start_time, end_time, timeduration, work_type as "work_type: models::WorktimeType" FROM worktime WHERE employee_id = $1"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}

// pub(crate) async fn start_timer(
//     employee_id: i32,
//     worktype: models::WorktimeType,
//     pool: &sqlx::Pool<sqlx::Postgres>,
// ) -> sqlx::Result<models::Worktime> {
//     sqlx::query_as!(models::Worktime, "INSERT INTO worktime(employee_id, start_time, work_type) values($1, $2, $3) RETURNING employee_id", employee_id, time_utils::create_timestamp(), worktype).fetch_one(pool).await
// }
//
#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures("../../fixtures/task.sql", "../../fixtures/address.sql", "../../fixtures/employee.sql", "../../fixtures/worktime.sql"))]
    async fn test_get_timers(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &get_timers(1, &pool).await?[0];

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(worktime.work_type, Some(models::WorktimeType::Break));
        assert_eq!(worktime.end_time, None);

        Ok(())
    }
}
