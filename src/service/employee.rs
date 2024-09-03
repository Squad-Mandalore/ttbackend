use sqlx::PgPool;

use crate::security::{create_salt, hash_password};

pub async fn update_password(pool: &PgPool, employee_id: &i32, new_password: String) -> sqlx::Result<()> {
    let salt_length = dotenvy::var("SALT_LENGTH")
        .expect("SALT_LENGTH variable not set.")
        .parse::<usize>()
        .expect("SALT_LENGTH is not a proper number.");
    create_salt(pool, employee_id, salt_length).await?;

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
