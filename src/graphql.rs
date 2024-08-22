use async_graphql::{extensions::Logger, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Extension;

mod timer;
mod world;

#[derive(MergedObject, Default)]
pub struct Query(timer::Timer, world::World);

#[derive(MergedObject, Default)]
pub struct Mutation(timer::TimerMutation);

pub type SchemaType = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(database_pool: sqlx::PgPool) -> SchemaType {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Logger)
        .data(database_pool)
        .finish()
}

pub async fn graphql_handler(
    axum::extract::Extension(email): axum::extract::Extension<String>,
    schema: Extension<SchemaType>,
    request: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(request.into_inner().data(email))
        .await
        .into()
}
