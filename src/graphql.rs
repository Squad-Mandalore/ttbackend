use async_graphql::{extensions::Logger, EmptyMutation, EmptySubscription, Schema};
use query::Query;

mod query;

pub type SchemaType = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> SchemaType {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .extension(Logger)
        .finish()
}
