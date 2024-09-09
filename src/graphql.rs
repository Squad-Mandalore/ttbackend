use async_graphql::{extensions::Logger, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Extension;

mod employee;
mod pdf;
mod task;
mod timer;

#[derive(MergedObject, Default)]
pub struct Query(timer::Timer, task::TaskQuery, pdf::PDFQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(
    timer::TimerMutation,
    task::TaskMutation,
    employee::EmployeeMutation,
);

pub type SchemaType = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(database_pool: sqlx::PgPool) -> SchemaType {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Logger)
        .data(database_pool)
        .finish()
}

pub async fn graphql_handler(
    axum::extract::Extension(employee_id): axum::extract::Extension<String>,
    schema: Extension<SchemaType>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(
            request
                .into_inner()
                .data(employee_id.parse::<i32>().unwrap()),
        )
        .await
        .into()
}
