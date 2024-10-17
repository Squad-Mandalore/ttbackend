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

pub async fn get_employee(employee_id: &i32, pool: &PgPool) -> sqlx::Result<models::Employee> {
    sqlx::query_as!(models::Employee, "SELECT employee_id, firstname, lastname, email, weekly_time, address_id FROM employee WHERE employee_id = $1", employee_id,).fetch_one(pool).await
}

pub async fn initial_password(employee_id: &i32, pool: &PgPool) -> sqlx::Result<bool> {
    let inital = sqlx::query!(
        "SELECT pw_salt FROM employee WHERE employee_id = $1",
        employee_id,
    )
    .fetch_one(pool)
    .await?
    .pw_salt
    .is_none();

    Ok(inital)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
    ))]
    async fn test_get_employee(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let employee = &get_employee(&1, &pool).await?;

        assert_eq!(employee.employee_id, 1);
        assert!(employee.firstname.is_some());
        assert_eq!(employee.firstname.as_ref().unwrap(), "bob");
        assert_eq!(employee.address_id, 1);

        Ok(())
    }

    #[sqlx::test(fixtures(
        "../../fixtures/truncate.sql",
        "../../fixtures/address.sql",
        "../../fixtures/employee.sql",
    ))]
    async fn test_initial_password(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let initial = &initial_password(&2, &pool).await?;

        assert!(initial);

        update_password("new_password".to_string(), &pool, &2).await?;
        let initial = &initial_password(&2, &pool).await?;

        assert!(!initial);

        Ok(())
    }
}
