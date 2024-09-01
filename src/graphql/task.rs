use async_graphql::Object;

use crate::{models, service};

pub struct TaskQuery;

#[Object]
impl TaskQuery {
    async fn tasks(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Vec<models::Task>> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        service::task::get_tasks(pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn task_by_id(
        &self,
        ctx: &async_graphql::Context<'_>,
        task_id: i32,
    ) -> async_graphql::Result<Option<models::Task>> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        service::task::get_task_by_id(task_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }
}

impl Default for TaskQuery {
    fn default() -> Self {
        TaskQuery
    }
}

pub struct TaskMutation;

#[Object]
impl TaskMutation {
    async fn create_task(
        &self,
        ctx: &async_graphql::Context<'_>,
        task_description: String,
    ) -> async_graphql::Result<models::Task> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        service::task::create_task(&task_description, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn update_task(
        &self,
        ctx: &async_graphql::Context<'_>,
        task_id: i32,
        task_description: String,
    ) -> async_graphql::Result<Option<models::Task>> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        service::task::update_task(task_id, &task_description, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }

    async fn delete_task(
        &self,
        ctx: &async_graphql::Context<'_>,
        task_id: i32,
    ) -> async_graphql::Result<Option<models::Task>> {
        let pool = ctx.data::<sqlx::PgPool>()?;

        service::task::delete_task(task_id, pool)
            .await
            .map_err(|e| async_graphql::Error::new_with_source(e))
    }
}

impl Default for TaskMutation {
    fn default() -> Self {
        TaskMutation
    }
}
