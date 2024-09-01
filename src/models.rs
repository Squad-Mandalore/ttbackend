use sqlx::postgres::types;

use crate::service;

#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq, Debug, sqlx::Type)]
#[sqlx(type_name = "worktime_type", rename_all = "lowercase")]
pub enum WorktimeType {
    Break,
    Ride,
    Work,
}

#[derive(async_graphql::SimpleObject, sqlx::FromRow)]
#[graphql(complex)]
pub struct Worktime {
    pub worktime_id: i32,
    pub employee_id: i32,
    pub task_id: i32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    #[graphql(skip)]
    pub timeduration: Option<types::PgInterval>,
    pub work_type: WorktimeType,
}

#[async_graphql::ComplexObject]
impl Worktime {
    async fn timeduration(&self) -> Option<String> {
        match &self.timeduration {
            None => None,
            Some(interval) => {
                let months = interval.months;
                let days = interval.days;
                let seconds = interval.microseconds as f64 / 1_000_000_f64;
                Some(format!("P{}M{}DT0H0M{}S", months, days, seconds))
            }
        }
    }

    async fn task(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Task> {
        let pool = ctx.data::<sqlx::PgPool>()?;
        service::task::get_task_by_id(self.task_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))?
            .ok_or(async_graphql::Error::new(format!(
                "Task with id '{}' could not be found.",
                self.task_id
            )))
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct Task {
    pub task_id: i32,
    pub task_description: Option<String>,
}
