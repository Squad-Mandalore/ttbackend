use crate::pdf::{generate_pdf, HeaderColor};

#[derive(Default)]
pub struct PDFQuery;

#[async_graphql::Object]
impl PDFQuery {
    async fn generate_pdf(
        &self,
        ctx: &async_graphql::Context<'_>,
        header_color: HeaderColor,
        month: String,
    ) -> async_graphql::Result<String> {
        let pool = ctx.data::<sqlx::Pool<sqlx::Postgres>>()?;
        let employee_id = ctx.data::<i32>()?;

        generate_pdf(month, header_color, pool, employee_id)
            .await
            .map_err(async_graphql::Error::new_with_source)
    }
}
