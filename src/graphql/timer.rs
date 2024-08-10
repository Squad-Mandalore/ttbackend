use crate::{models, service::worktime};

pub struct Timer;

#[async_graphql::Object]
impl Timer {
    async fn timers(
        &self,
        ctx: &async_graphql::Context<'_>,
        employee_id: i32,
    ) -> async_graphql::Result<Vec<models::Worktime>> {
        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;

        worktime::get_timers(employee_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }
}

impl Default for Timer {
    fn default() -> Self {
        Timer
    }
}
