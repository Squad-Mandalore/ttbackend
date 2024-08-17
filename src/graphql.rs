use async_graphql::{extensions::Logger, EmptyMutation, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Extension;
use sqlx::{Pool, Postgres};

mod hello;
mod world;

#[derive(MergedObject, Default)]
pub struct Query(hello::Hello, world::World);

pub type SchemaType = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema(database_pool: Pool<Postgres>) -> SchemaType {
    Schema::build(Query::default(), EmptyMutation, EmptySubscription)
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
