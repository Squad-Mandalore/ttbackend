use crate::{
    models,
    service::employee::{get_employee, update_password},
};

#[derive(Default)]
pub struct EmployeeMutation;

#[async_graphql::Object]
impl EmployeeMutation {
    async fn update_password(
        &self,
        ctx: &async_graphql::Context<'_>,
        new_password: String,
    ) -> async_graphql::Result<models::Employee> {
        let pool = ctx.data::<sqlx::PgPool>()?;
        let employee_id = ctx.data::<i32>()?;

        update_password(new_password, pool, employee_id)
            .await
            .map_err(async_graphql::Error::new_with_source)
    }
}

#[derive(Default)]
pub struct EmployeeQuery;

#[async_graphql::Object]
impl EmployeeQuery {
    async fn get_employee(
        &self,
        ctx: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<models::Employee> {
        let pool = ctx.data::<sqlx::PgPool>()?;
        let employee_id = ctx.data::<i32>()?;

        get_employee(employee_id, pool)
            .await
            .map_err(async_graphql::Error::new_with_source)
    }
}
