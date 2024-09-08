use sqlx::PgPool;

use crate::security::{create_salt, hash_password};

pub async fn update_password(
    new_password: String,
    pool: &PgPool,
    employee_id: &i32,
) -> sqlx::Result<()> {
    create_salt(pool, employee_id).await?;

    let hashed_password = hash_password(new_password, pool, employee_id).await?;
    let _ = sqlx::query!(
        "UPDATE employee SET password = $2 WHERE employee_id = $1",
        employee_id,
        hashed_password,
    )
    .execute(pool)
    .await?;

    Ok(())
}
