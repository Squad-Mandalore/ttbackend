use crate::{models, service::worktime};

pub struct Timer;

#[async_graphql::Object]
impl Timer {
    async fn timers(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<models::Worktime>> {
        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;
        let employee_id = ctx.data::<i32>()?;

        worktime::get_timers(employee_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn timers_in_boundary(
        &self,
        ctx: &async_graphql::Context<'_>,
        lower_bound: chrono::DateTime<chrono::FixedOffset>,
        upper_bound: chrono::DateTime<chrono::FixedOffset>,
    ) -> async_graphql::Result<Vec<models::Worktime>> {
        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;
        let employee_id = ctx.data::<i32>()?;

        worktime::get_timers_in_boundary(employee_id, lower_bound, upper_bound, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn timers_today(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<models::Worktime>> {
        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;
        let employee_id = ctx.data::<i32>()?;

        let now = chrono::Utc::now().fixed_offset();

        let lower_bound = now.with_time(chrono::NaiveTime::MIN).single().unwrap();

        let upper_bound = lower_bound.checked_add_days(chrono::Days::new(1)).unwrap();

        worktime::get_timers_in_boundary(employee_id, lower_bound, upper_bound, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn timers_current_month(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<models::Worktime>> {
        use chrono::Datelike;

        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;
        let employee_id = ctx.data::<i32>()?;

        let now = chrono::Utc::now().fixed_offset().with_day(1).unwrap();

        let lower_bound = now.with_time(chrono::NaiveTime::MIN).single().unwrap();

        let upper_bound = lower_bound
            .checked_add_months(chrono::Months::new(1))
            .unwrap();

        worktime::get_timers_in_boundary(employee_id, lower_bound, upper_bound, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }
}

impl Default for Timer {
    fn default() -> Self {
        Timer
    }
}

pub struct TimerMutation;

#[async_graphql::Object]
impl TimerMutation {
    async fn start_timer(
        &self,
        ctx: &async_graphql::Context<'_>,
        task_id: i32,
        worktype: models::WorktimeType,
    ) -> async_graphql::Result<models::Worktime> {
        let pool = ctx.data::<sqlx::PgPool>()?;
        let employee_id = ctx.data::<i32>()?;

        worktime::start_timer(employee_id, task_id, worktype, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn stop_timer(
        &self,
        ctx: &async_graphql::Context<'_>,
        worktime_id: i32,
    ) -> async_graphql::Result<models::Worktime> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        worktime::stop_timer(worktime_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn update_timer(
        &self,
        ctx: &async_graphql::Context<'_>,
        worktime_id: i32,
        task_id: Option<i32>,
        start_time: Option<chrono::DateTime<chrono::FixedOffset>>,
        end_time: Option<chrono::DateTime<chrono::FixedOffset>>,
        worktype: Option<models::WorktimeType>,
    ) -> async_graphql::Result<models::Worktime> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        worktime::update_timer(worktime_id, task_id, start_time, end_time, worktype, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }
}

impl Default for TimerMutation {
    fn default() -> Self {
        TimerMutation
    }
}
