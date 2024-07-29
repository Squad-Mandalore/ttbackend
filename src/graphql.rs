use async_graphql::{extensions::Logger, EmptyMutation, EmptySubscription, MergedObject, Schema};

mod hello;
mod logs;

#[derive(MergedObject, Default)]
pub struct Query(hello::Hello, logs::Logs);

pub type SchemaType = Schema<Query, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> SchemaType {
    Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .extension(Logger)
        .finish()
}
