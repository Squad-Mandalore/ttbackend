use sqlx::PgPool;

use crate::{
    models,
    security::{create_salt, hash_password},
};

pub async fn update_password(
    new_password: String,
    pool: &PgPool,
    employee_id: &i32,
) -> sqlx::Result<models::Employee> {
    create_salt(pool, employee_id).await?;

    let hashed_password = hash_password(new_password, pool, employee_id).await?;
    sqlx::query_as!(
        models::Employee,
        "UPDATE employee SET password = $2 WHERE employee_id = $1 RETURNING employee_id, firstname, lastname, email, weekly_time, address_id",
        employee_id,
        hashed_password,
    )
    .fetch_one(pool)
    .await
}
