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
    task_description: &str,
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
    task_description: &str,
    pool: &sqlx::PgPool,
) -> sqlx::Result<Option<models::Task>> {
    sqlx::query_as!(
        models::Task,
        "UPDATE task SET task_description = $2 WHERE task_id = $1 RETURNING task_id, task_description",
        task_id,
        task_description
    )
    .fetch_optional(pool)
    .await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures("../../fixtures/truncate.sql", "../../fixtures/task.sql",))]
    async fn test_get_task_by_id(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &get_task_by_id(1, &pool).await?;

        assert!(task.is_some());
        let task = task.as_ref().unwrap();
        assert_eq!(task.task_id, 1);
        assert_ne!(task.task_description, None);
        assert_eq!(task.task_description.as_ref().unwrap(), "first task");

        Ok(())
    }

    #[sqlx::test(fixtures("../../fixtures/truncate.sql", "../../fixtures/task.sql",))]
    async fn test_can_not_get_task_by_id(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &get_task_by_id(10, &pool).await?;

        assert!(task.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("../../fixtures/truncate.sql", "../../fixtures/task.sql",))]
    async fn test_get_tasks(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &get_tasks(&pool).await?;

        assert_eq!(task.len(), 2);
        let task = &task[0];

        assert_eq!(task.task_id, 1);
        assert_ne!(task.task_description, None);
        assert_eq!(task.task_description.as_ref().unwrap(), "first task");

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_task(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &create_task("test", &pool).await?;

        assert_eq!(task.task_id, 1);
        assert_ne!(task.task_description, None);
        assert_eq!(task.task_description.as_ref().unwrap(), "test");

        Ok(())
    }

    #[sqlx::test(fixtures("../../fixtures/truncate.sql", "../../fixtures/task.sql",))]
    async fn test_update_task(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &update_task(1, "test", &pool).await?;

        assert!(task.is_some());
        let task = task.as_ref().unwrap();

        assert_eq!(task.task_id, 1);
        assert_ne!(task.task_description, None);
        assert_eq!(task.task_description.as_ref().unwrap(), "test");

        Ok(())
    }

    #[sqlx::test(fixtures("../../fixtures/truncate.sql", "../../fixtures/task.sql",))]
    async fn test_delete_task(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let task = &delete_task(1, &pool).await?;

        assert!(task.is_some());
        let task = task.as_ref().unwrap();

        assert_eq!(task.task_id, 1);
        assert_ne!(task.task_description, None);
        assert_eq!(task.task_description.as_ref().unwrap(), "first task");

        Ok(())
    }
}
