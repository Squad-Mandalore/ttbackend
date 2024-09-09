use crate::{models, service::employee::update_password};

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

impl Default for EmployeeMutation {
    fn default() -> Self {
        EmployeeMutation
    }
}
