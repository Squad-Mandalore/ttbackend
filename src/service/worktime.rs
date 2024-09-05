use sqlx::query_builder;

use crate::models;

pub async fn get_timers(
    employee_id: &i32,
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

pub async fn get_timers_in_boundary(
    employee_id: &i32,
    lower_bound: chrono::DateTime<chrono::FixedOffset>,
    upper_bound: chrono::DateTime<chrono::FixedOffset>,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Vec<models::Worktime>> {
    sqlx::query_as!(
        models::Worktime,
        r#"SELECT worktime_id, employee_id, task_id, start_time, end_time, timeduration, work_type as "work_type: models::WorktimeType" FROM worktime WHERE employee_id = $1 AND start_time >= $2 AND start_time < $3"#,
        employee_id,
        lower_bound,
        upper_bound,
    )
    .fetch_all(pool)
    .await
}

pub(crate) async fn start_timer(
    employee_id: &i32,
    task_id: i32,
    worktype: models::WorktimeType,
    pool: &sqlx::PgPool,
) -> sqlx::Result<models::Worktime> {
    sqlx::query_as!(
        models::Worktime,
        r#"
        INSERT INTO worktime(employee_id, task_id, work_type)
        VALUES($1, $2, $3)
        RETURNING worktime_id, employee_id, task_id, start_time, end_time, timeduration, work_type as "work_type: models::WorktimeType"
        "#,
        employee_id,
        task_id,
        worktype as models::WorktimeType
    )
    .fetch_one(pool)
    .await
}

pub(crate) async fn stop_timer(
    worktime_id: i32,
    pool: &sqlx::PgPool,
) -> sqlx::Result<models::Worktime> {
    sqlx::query_as!(
        models::Worktime,
        r#"
        UPDATE worktime
        SET end_time = NOW()
        WHERE worktime_id = $1
        RETURNING worktime_id, employee_id, task_id, start_time, end_time, timeduration, work_type as "work_type: models::WorktimeType"
        "#,
        worktime_id,
    )
    .fetch_one(pool)
    .await
}

pub(crate) async fn update_timer(
    worktime_id: i32,
    task_id: Option<i32>,
    start_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    end_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    worktype: Option<models::WorktimeType>,
    pool: &sqlx::PgPool,
) -> sqlx::Result<models::Worktime> {
    let mut query_builder =
        query_builder::QueryBuilder::<sqlx::Postgres>::new("UPDATE worktime SET ");

    let mut needs_comma = false;

    if let Some(task_id) = task_id {
        if needs_comma {
            query_builder.push(", ");
        }
        query_builder.push("task_id = ").push_bind(task_id);
        needs_comma = true;
    }

    if let Some(start_time) = start_time {
        if needs_comma {
            query_builder.push(", ");
        }
        query_builder.push("start_time = ").push_bind(start_time);
        needs_comma = true;
    }

    if let Some(end_time) = end_time {
        if needs_comma {
            query_builder.push(", ");
        }
        query_builder.push("end_time = ").push_bind(end_time);
        needs_comma = true;
    }

    if let Some(worktype) = worktype {
        if needs_comma {
            query_builder.push(", ");
        }
        query_builder
            .push("work_type = ")
            .push_bind(worktype as models::WorktimeType);
        needs_comma = true;
    }

    if !needs_comma {
        return Err(sqlx::Error::RowNotFound); // No fields were provided to update
    }

    let query = query_builder
        .push(" WHERE worktime_id = ")
        .push_bind(worktime_id)
        .push(" RETURNING worktime_id, employee_id, task_id, start_time, end_time, timeduration, work_type")
        .build_query_as::<models::Worktime>();

    query.fetch_one(pool).await
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use sqlx::postgres::types::PgInterval;

    use super::*;

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_get_timers(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &get_timers(&1, &pool).await?[0];

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T08:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T16:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 8 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql"
    ))]
    async fn test_start_timer(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &start_timer(&1, 1, models::WorktimeType::Break, &pool).await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(worktime.end_time, None);
        assert_eq!(worktime.timeduration, None);
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_stop_timer(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &stop_timer(2, &pool).await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_ne!(worktime.end_time, None);
        assert_ne!(worktime.timeduration, None);
        assert_eq!(worktime.work_type, models::WorktimeType::Work);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_update_timer_task(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &update_timer(1, Some(2), None, None, None, &pool).await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 2);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T08:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T16:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 8 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_update_timer_start(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &update_timer(
            1,
            None,
            chrono::DateTime::parse_from_rfc3339("2024-01-01T09:00:00+00:00").ok(),
            None,
            None,
            &pool,
        )
        .await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T09:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T16:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 7 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_update_timer_end(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &update_timer(
            1,
            None,
            None,
            chrono::DateTime::parse_from_rfc3339("2024-01-01T15:00:00+00:00").ok(),
            None,
            &pool,
        )
        .await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T08:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T15:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 7 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_update_timer_worktype(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime =
            &update_timer(1, None, None, None, Some(models::WorktimeType::Ride), &pool).await?;

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T08:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T16:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 8 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Ride);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/task.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
        "../../fixtures/worktime.sql"
    ))]
    async fn test_get_timers_in_boundary(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let worktime = &get_timers_in_boundary(
            &1,
            chrono::DateTime::from_str("2024-01-01T00:00:00Z").unwrap(),
            chrono::DateTime::from_str("2024-01-02T00:00:00Z").unwrap(),
            &pool,
        )
        .await?;

        assert_eq!(worktime.len(), 1);

        let worktime = &worktime[0];

        assert_eq!(worktime.employee_id, 1);
        assert_eq!(worktime.task_id, 1);
        assert_eq!(
            worktime.start_time.to_rfc3339(),
            "2024-01-01T08:00:00+00:00"
        );
        assert_ne!(worktime.end_time, None);
        assert_eq!(
            worktime.end_time.unwrap().to_rfc3339(),
            "2024-01-01T16:00:00+00:00"
        );
        assert_ne!(worktime.timeduration, None);
        let secs: u64 = 8 * 60 * 60;
        assert_eq!(
            worktime.timeduration.clone().unwrap(),
            PgInterval::try_from(std::time::Duration::from_secs(secs)).unwrap()
        );
        assert_eq!(worktime.work_type, models::WorktimeType::Break);

        Ok(())
    }
}
