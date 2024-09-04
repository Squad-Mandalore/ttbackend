use sqlx::PgPool;

use crate::security::{create_salt, hash_password};

pub async fn update_password(
    pool: &PgPool,
    employee_id: &i32,
    new_password: String,
) -> sqlx::Result<()> {
    create_salt(pool, employee_id).await?;

    let hashed_password = hash_password(pool, employee_id, new_password).await?;
    let _ = sqlx::query!(
        "UPDATE employee SET password = $2 WHERE employee_id = $1",
        employee_id,
        hashed_password,
    )
    .execute(pool)
    .await?;

    Ok(())
}
